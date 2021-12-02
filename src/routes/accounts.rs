use actix_web::{web, HttpResponse};
use sqlx::PgPool;

#[derive(serde::Deserialize, Debug)]
pub struct AccountCreationData {
    email: String,
    username: String,
    password: String,
}

#[derive(serde::Serialize)]
struct AccountCreationResponse {
    message: String
}

#[tracing::instrument(
    name = "Adding a new user",
    skip(data, connection_pool),
    fields(
        account_email = %data.email,
        account_username = %data.username
    )
)]
pub async fn create_account(
    data: web::Form<AccountCreationData>,
    connection_pool: web::Data<PgPool>,
) -> HttpResponse {
    match insert_account(&data, &connection_pool).await
    {
        Ok(_) => HttpResponse::Ok().json(
            AccountCreationResponse {
            message: "New account successfully created".into()
        }),
        Err(e) => HttpResponse::InternalServerError().json(AccountCreationResponse {
            message: format!("Failed to create account {:#}", e)
        })
    }
}

#[tracing::instrument(
    name = "Saving new account details",
    skip(data, connection_pool),
)]    
pub async fn insert_account(
    data: &AccountCreationData,
    connection_pool: &PgPool,    
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO accounts (id, email, username, password, created_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        uuid::Uuid::new_v4(),
        data.email,
        data.username,
        data.password,
        chrono::Utc::now()
    )
    .execute(connection_pool)
    // .instrument(query_span)
    .await
    .map_err(|e| {
        e
    })?;

    Ok(())
}

