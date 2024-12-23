use std::net::TcpListener;
use actix_web::{
    dev::Server, web::{self, Data}, App, HttpServer 
};
use tracing_actix_web::TracingLogger;
use crate::email_client::{self, EmailClient};
use crate::{in_memory::AppState, routes::{greet, health_check, subscribe}};

pub fn run(listener: TcpListener, 
    app_state:web::Data<AppState>,
    email_client: EmailClient) -> Result<Server, std::io::Error> {
    let email_client = Data::new(email_client);
    let server = HttpServer::new(move|| {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(app_state.clone())
            .app_data(email_client.clone()) // each app will get a shared reference to same client (to use the same connection pool created by reqwest under the hood)
            .route("/health_check", web::get().to(health_check))
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
            .route("/subscriptions", web::post().to(subscribe))
            
    })
    .listen(listener)?
    .run();

    Ok(server)
}
