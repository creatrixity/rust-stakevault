use crate::email_client::EmailClient;
use crate::configuration::{ Settings, DatabaseSettings, ApplicationBaseUrl };
use crate::routes;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn run(listener: TcpListener, connection_pool: PgPool, email_client: EmailClient, base_url: String) -> Result<Server, std::io::Error> {
    let connection_pool = web::Data::new(connection_pool);
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(routes::health_check))
            .route("/subscriptions", web::post().to(routes::subscribe))
            .route("/subscriptions/confirm", web::get().to(routes::confirm))
            .route("/account/create", web::post().to(routes::create_account))
            .app_data(connection_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

pub struct Application {
    port: u16,
    server: Server
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);
        let sender_email = configuration.email_client.sender().expect("Failed to read email client sender email address");
        let timeout_milliseconds = configuration.email_client.timeout_milliseconds;
    
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            std::time::Duration::from_millis(timeout_milliseconds)
        );
    
        let address: &str = &format!("{}:{}", configuration.application.host, configuration.application.port);
        let listener = std::net::TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
    
        let server = run(listener, connection_pool, email_client, configuration.application.base_url)?;

        Ok(Self {
            server,
            port
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
    .connect_timeout(std::time::Duration::from_secs(2))    
    .connect_lazy_with(configuration.with_db())
}