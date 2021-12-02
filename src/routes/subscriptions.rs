use actix_web::{HttpResponse, web::{self}};
use sqlx::PgPool;
use crate::email_client::{self, EmailClient};
use std::convert::{ TryFrom, TryInto };
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
    match insert_subscriber(&new_subscriber, &connection_pool).await
    {
        Ok(_) => {
            if send_confirmation_email(&email_client, new_subscriber, &base_url.0).await.is_err() {
                return HttpResponse::InternalServerError().json(SubscriberCreationResponse {
                    message: "Failed to send email {:#}".to_string()
                })
            }

            HttpResponse::Ok().json(
                SubscriberCreationResponse {
                message: "New subscriber successfully created".into()
            })
        },
        Err(e) => HttpResponse::InternalServerError().json(SubscriberCreationResponse {
            message: format!("Failed to create subscriber {:#}", e)
        })
    }
}

#[tracing::instrument(
    name = "Send a confirmation email to a new ",
    skip(email_client, new_subscriber),
)]
pub async fn send_confirmation_email (
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!("{}/subscriptions/confirm?subscription_token=mytoken", base_url);

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
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, created_at, status)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        uuid::Uuid::new_v4(),
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

    Ok(())
}

