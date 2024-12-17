use std::net::TcpListener;
use env_logger::Env;
use zero2prod::{configuration::get_configuration, startup::run};
use zero2prod::in_memory::AppState;
use actix_web::web;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // `init` does call `set_logger`, so this is all we need to do.
    // We are falling back to printing all logs at info-level or above
    // if the RUST_LOG environment variable has not been set.
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    // create the in-memory list for sharing accross threads
    let app_state: AppState = AppState::new();
    let data_store_shared = web::Data::new(app_state);
    // creating the properties data from our configuration.yaml file
    let configuration = get_configuration().expect("Failed to read configuration");
    // building the servers adress and port
    let address = format!("127.0.0.1:{}", configuration.server_port);
    // binding to port
    let listener = TcpListener::bind(address).expect("Binding to port failed");
    // running the receiving http server
    run(listener, data_store_shared)?.await
}
