use std::net::TcpListener;
use zero2prod::in_memory::AppState;
use actix_web::web;

pub fn spawn_app(listener: TcpListener) {
    let app_state: AppState = AppState::new();
    let data_store_shared = web::Data::new(app_state);
    let server = zero2prod::startup::run(listener, data_store_shared).expect("Failed to bind address");
    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it here, hence the non-binding let
    let _ = tokio::spawn(server);
}