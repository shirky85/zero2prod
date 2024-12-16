use std::net::TcpListener;
use zero2prod::{configuration::get_configuration, startup::run};
use zero2prod::in_memory::AppState;
use actix_web::web;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app_state: AppState = AppState::new();
    let data_store_shared = web::Data::new(app_state);
    let configuration = get_configuration().expect("Failed to read configuration");
    let address = format!("127.0.0.1:{}", configuration.server_port);
    let listener = TcpListener::bind(address).expect("Binding to port failed");
    // Arrange
    run(listener, data_store_shared)?.await
}
