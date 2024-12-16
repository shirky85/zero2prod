use std::net::TcpListener;
use zero2prod::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = get_configuration().expect("Failed to read configuration");
    let address = format!("127.0.0.1:{}", configuration.server_port);
    let listener = TcpListener::bind(address).expect("Binding to port failed");
    // Arrange
    run(listener)?.await
}
