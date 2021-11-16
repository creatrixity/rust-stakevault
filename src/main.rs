use rust_stakevault::configuration::get_configuration;
use rust_stakevault::startup::run;
use sqlx::PgPool;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read application configuration");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to PostgreSQL");

    let address: &str = &format!("127.0.0.1:{}", configuration.application_port);
    let listener = std::net::TcpListener::bind(address)?;

    run(listener, connection_pool)?.await
}
