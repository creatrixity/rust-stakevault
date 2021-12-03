use actix_web::{HttpResponse, web::{self}};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::PgPool;
use crate::email_client::{EmailClient};
use std::{convert::{ TryFrom, TryInto }};
use crate::domain::{ NewSubscriber, SubscriberName, SubscriberEmail };
use crate::configuration::ApplicationBaseUrl;

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String,
}

#[derive(serde::Serialize)]
struct SubscriberCreationResponse {
    message: String
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(form.name)?;
        let email = SubscriberEmail::parse(form.email)?;
    
        Ok(Self {
            email,
            name
        })
    }    
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();

    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}


#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(data, connection_pool, base_url),
    fields(
        subscriber_email = %data.email,
        subscriber_name = %data.name
    )
)]
pub async fn subscribe(
    data: web::Form<FormData>,
    connection_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>
) -> HttpResponse {
    let new_subscriber = match data.0.try_into() {
        Ok(data) => data,
        Err(_) => return HttpResponse::BadRequest().finish()
    };
   let subscriber_id = match insert_subscriber(&new_subscriber, &connection_pool).await
    {
        Ok(subscriber_id) => subscriber_id,
        Err(e) => return HttpResponse::InternalServerError().json(SubscriberCreationResponse {
            message: format!("Failed to create subscriber {:#}", e)
        })
    };

    let subscription_token = generate_subscription_token();
    if store_token(&connection_pool, subscriber_id, &subscription_token).await.is_err() {
        return HttpResponse::InternalServerError().finish()
    }
    
    if send_confirmation_email(&email_client, new_subscriber, &base_url.0, &subscription_token).await.is_err() {
        return HttpResponse::InternalServerError().json(SubscriberCreationResponse {
            message: "Failed to send email {:#}".to_string()
        })
    }

    HttpResponse::Ok().json(
        SubscriberCreationResponse {
        message: "New subscriber successfully created".into()
    })

}

#[tracing::instrument(
    name = "Store confirmation nonce for a new subscriber",
    skip(pool, subscription_token)
)]
pub async fn store_token(pool: &PgPool, subscriber_id: uuid::Uuid, subscription_token: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO subscription_tokens (subscription_token, subscriber_id)
            VALUES ($1, $2)
        "#,
        subscription_token,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute: {:?}", e);

        e
    })?;

    Ok(())
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber),
)]
pub async fn send_confirmation_email (
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    token: &str
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!("{}/subscriptions/confirm?subscription_token={}", base_url, token);

    email_client.send_email(
        new_subscriber.email,
        "Welcome!",
        &format!("Welcome to our app \n Visit {} to confirm your subscription", confirmation_link),
        &format!(
            "Welcome to our app <br /> \
            Click <a href=\"{}\">here</a> to confirm your subscription
        ", confirmation_link)
    ).await
}


#[tracing::instrument(
    name = "Saving new subscriber details",
    skip(data, connection_pool),
)]    
pub async fn insert_subscriber(
    data: &NewSubscriber,
    connection_pool: &PgPool,    
) -> Result<uuid::Uuid, sqlx::Error> {
    let id = uuid::Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, created_at, status)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        id,
        data.email.as_ref(),
        data.name.as_ref(),
        chrono::Utc::now(),
        "pending_confirmation"
    )
    .execute(connection_pool)
    .await
    .map_err(|e| {
        e
    })?;

    Ok(id)
}

