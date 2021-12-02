use rust_stakevault::configuration::{get_configuration};
use rust_stakevault::startup::{ Application };
use rust_stakevault::telemetry::{get_subscriber, init_subscriber};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create tracing subscriber and initialize it
    let subscriber = get_subscriber("stakevault".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read application configuration");
    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;

    Ok(())
}
