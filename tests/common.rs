use std::net::TcpListener;

pub fn spawn_app(listener: TcpListener) {
    let server = zero2prod::startup::run(listener).expect("Failed to bind address");
    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it here, hence the non-binding let
    let _ = tokio::spawn(server);
}