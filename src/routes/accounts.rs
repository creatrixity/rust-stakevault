use actix_web::{web, HttpResponse};
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct AccountCreationData {
    email: String,
    username: String,
    password: String,
}

pub async fn create_account(
    data: web::Form<AccountCreationData>,
    connection_pool: web::Data<PgPool>,
) -> HttpResponse {
    match sqlx::query!(
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
    .execute(connection_pool.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
