use serde_json::Value;
use zero2prod::routes::SubscriptionRequest;
use zero2prod::startup::Application;
use wiremock::{matchers::{method, path}, Mock, MockServer, ResponseTemplate};
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
    pub port: u16,
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: &SubscriptionRequest) -> reqwest::Response {
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

    pub async fn get_subscription(&self, subscription_id: &str) -> reqwest::Response {
        reqwest::Client::new()
            .get(&format!("{}/subscriptions/find?subscription_id={}", &self.address, subscription_id))
            .header("Content-Type", "application/json")
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn get_confirmation_links(
        &self,
        email_request: &wiremock::Request
        ) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(
            &email_request.body
            ).unwrap();
        // Extract the link from one of the request fields.
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            // Let's make sure we don't call random APIs on the web
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };
        let html = get_link(&body["Html-part"].as_str().unwrap());
        let plain_text = get_link(&body["Text-part"].as_str().unwrap());
        ConfirmationLinks {
            html,
            plain_text
        }
    }

    pub async fn create_unconfirmed_subscription(&self) -> ConfirmationLinks{
        let _mock_send_confirmation = Mock::given(path("/v3/send"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .named("Send confirmation email")
            .mount_as_scoped(&self.mock_email_server) // this is a mockGuard object - 
            //it will mock according to the specification only inside the scope it was declared in
            .await;

        let _unconfirmed_subscription_response = self.post_subscriptions(
            &SubscriptionRequest::new("le guin".to_string(), 
            "ursula_le_guin@gmail.com".to_string())).await.error_for_status().unwrap();

        //catch the request that was sent to the mock email server for the confirmation email
        let email_request = &self
        .mock_email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
        
        self.get_confirmation_links(&email_request)
            
    }

    pub async fn create_confirmed_subscription(&self) {
        let confirmation_link = self.create_unconfirmed_subscription().await;

        // Now we send the confirmation request
        let _response = reqwest::get(confirmation_link.html).await.unwrap();
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
    let port = application.port();
    let address = format!("http://127.0.0.1:{}", port);
    let _ = tokio::spawn(application.run_until_stopped());

    TestApp { 
        address: address,
        mock_email_server: email_server,
        port: port,
     }
}

pub fn get_id_from_response(response_body: String) -> String{
    let json: Value = serde_json::from_str(&response_body).unwrap();

    // Extract the "id" field
    json["id"].as_u64()
                                .map(|id| id.to_string())
                                .unwrap_or_else(|| "".to_string())
}