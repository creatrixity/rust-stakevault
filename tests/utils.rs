use once_cell::sync::Lazy;
use rust_stakevault::configuration::{ DatabaseSettings, get_configuration};
use sqlx::{ Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

use rust_stakevault::telemetry::{get_subscriber, init_subscriber};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "debug".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub connection_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to a random port");
    let port = listener.local_addr().unwrap().port();
    let connection_pool = configure_database(&configuration.database).await;
    let server = rust_stakevault::startup::run(listener, connection_pool.clone())
    .expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp {
        connection_pool,
        address: format!("http://127.0.0.1:{}", port),
    }
}

pub async fn configure_database(configuration: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&configuration.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");
        
    connection.execute(format!(r#"CREATE DATABASE "{}""#, configuration.database_name).as_str())
        .await
        .expect("Failed to create PGSQL database");

    let connection_pool = PgPool::connect(&configuration.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to run migrations on database");

    connection_pool
}
