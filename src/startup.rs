use actix_web::{ web, App, HttpServer };
use actix_web::dev::Server;
use std::net::TcpListener;
use crate::routes;

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(routes::health_check))
            .route("/account/create", web::post().to(routes::create_account))
    })
    .listen(listener)?
    .run();

    Ok(server)
}