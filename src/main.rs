use rust_stakevault::configuration::get_configuration;
use rust_stakevault::startup::run;
use rust_stakevault::telemetry::{get_subscriber, init_subscriber};
use sqlx::postgres::PgPoolOptions;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create tracing subscriber and initialize it
    let subscriber = get_subscriber("stakevault".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read application configuration");
    let connection_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))    
        .connect_lazy_with(configuration.database.with_db());

    let address: &str = &format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = std::net::TcpListener::bind(address)?;

    run(listener, connection_pool)?.await
}
