use actix_web::{ web, HttpResponse };

#[derive(serde::Deserialize)]
pub struct AccountCreationData {
    email: String,
    username: String,
    password: String
}

pub async fn create_account(_data: web::Form<AccountCreationData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
