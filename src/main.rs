use std::net::TcpListener;
use zero2prod::email_client::EmailClient;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod::{configuration::get_configuration, startup::run};
use zero2prod::in_memory::AppState;
use actix_web::web;



#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".into(), 
        "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    
    let app_state: AppState = AppState::new();
    let data_store_shared = web::Data::new(app_state);
    // creating the properties data from our configuration.yaml file
    let configuration = get_configuration().expect("Failed to read configuration");
    // building the servers adress and port
    let address = format!("127.0.0.1:{}", configuration.server_port);
    // binding to port
    let listener = TcpListener::bind(address).expect("Binding to port failed");
    // create a reqwest email client to inject to app data while taking details from configuration file
    let email_client = EmailClient::new(configuration.email_client.base_url, configuration.email_client.sender);
    // running the receiving http server
    run(listener, data_store_shared, email_client)?.await
}
