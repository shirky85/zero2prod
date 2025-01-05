use std::net::TcpListener;
use actix_web::{
    dev::Server, web::{self, Data}, App, HttpServer 
};
use tracing_actix_web::TracingLogger;
use sha3::Digest;
use crate::{email_client::EmailClient, in_memory::Sender, routes::{get_subscription, publish_newsletter, subscription_confirm}};
use crate::configuration::Properties;
use crate::{in_memory::AppState, routes::{greet, health_check, subscribe}};


pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    // We have converted the `build` function into a constructor for
    // `Application`.
    pub async fn build(configuration: Properties) -> Result<Self, std::io::Error> {
        let app_state: AppState = AppState::new();
        // TODO: Remove this hardcoded sender after writing an endpoint to register one
        app_state.senders.write().unwrap().push(Sender{
            username: "admin".to_string(),
            pwd: format!("{:x}",sha3::Sha3_256::digest("admin".as_bytes())),
        });
        let data_store_shared = web::Data::new(app_state);

        let sender_email = configuration
            .email_client
            .sender;
        
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
        );
        let address = format!(
            "{}:{}",
            configuration.server_host, configuration.server_port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, data_store_shared, email_client, configuration.base_url)?;
        // We "save" the bound port in one of `Application`'s fields
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }
    
    // A more expressive name that makes it clear that
    // this function only returns when the application is stopped.
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub struct ApplicationBaseUrl(pub String);

pub fn run(listener: TcpListener, 
    app_state:web::Data<AppState>,
    email_client: EmailClient,
    base_url_str: String) -> Result<Server, std::io::Error> {
    let email_client = Data::new(email_client);
    let base_url = Data::new(ApplicationBaseUrl(base_url_str));
    let server = HttpServer::new(move|| {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(app_state.clone())
            .app_data(email_client.clone()) // each app will get a shared reference to same client (to use the same connection pool created by reqwest under the hood)
            .app_data(base_url.clone())
            .route("/health_check", web::get().to(health_check))
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
            .route("/subscriptions/find",web::get().to(get_subscription))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(subscription_confirm))
            .route("/newsletters", web::post().to(publish_newsletter))
            
            
    })
    .listen(listener)?
    .run();

    Ok(server)
}
