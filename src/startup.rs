use std::net::TcpListener;
use actix_web::{
    dev::Server, web::{self}, App, HttpServer 
};
use tracing_actix_web::TracingLogger;

use crate::{in_memory::AppState, routes::{greet, health_check, subscribe}};

pub fn run(listener: TcpListener, app_state:web::Data<AppState>) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move|| {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(app_state.clone())
            .route("/health_check", web::get().to(health_check))
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
            .route("/subscriptions", web::post().to(subscribe))
            
    })
    .listen(listener)?
    .run();

    Ok(server)
}
