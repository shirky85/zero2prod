use zero2prod::routes::SubscriptionRequest;
use zero2prod::startup::Application;
use wiremock::MockServer;
use actix_web::web;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use std::sync::LazyLock;
use zero2prod::configuration::get_configuration;

/// The closure passed to LazyLock::new is executed only once, when TRACING is first accessed.
/// This ensures that the tracing stack is initialized only once.
/// It is is thread-safe, meaning it can be safely accessed from multiple threads without 
/// fear of multiple initializations.
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    // We cannot assign the output of `get_subscriber` to a variable based on the
    // value TEST_LOG` because the sink is part of the type returned by
    // `get_subscriber`, therefore they are not the same type. We could work around
    // it, but this is the most straight-forward way of moving forward.
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(
            subscriber_name,
            default_filter_level,
            std::io::stdout
        );
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(
            subscriber_name,
            default_filter_level,
            std::io::sink
        );
        init_subscriber(subscriber);
    };
});

pub struct TestApp{
    pub address: String,
    pub mock_email_server: MockServer,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: SubscriptionRequest) -> reqwest::Response {
        let request = web::Json(body);
        let json_payload = serde_json::to_string(&request).unwrap();
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/json")
            .body(json_payload)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub async fn spawn_app() -> TestApp {
    // This line forces the initialization of the TRACING variable if it has not already been initialized.
    // If we call it again later on in the code - it will do nothing.
    LazyLock::force(&TRACING);
    // Launch a mock server to stand in for Postmark's API
    let email_server = MockServer::start().await;
    // Randomise configuration to ensure test isolation
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        // Use a random OS port
        c.server_port = 0;
        // Use the mock server as email API
        c.email_client.base_url = email_server.uri();
        c
    };
    // let app_state: AppState = AppState::new();
    // let data_store_shared = web::Data::new(app_state);
    // let email_client = EmailClient::new(configuration.email_client.base_url, configuration.email_client.sender);
    // let server = zero2prod::startup::run(listener, data_store_shared, email_client).expect("Failed to bind address");
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    let address = format!("http://127.0.0.1:{}", application.port());
    let _ = tokio::spawn(application.run_until_stopped());

    TestApp { 
        address: address,
        mock_email_server: email_server,
     }
}