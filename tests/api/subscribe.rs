use reqwest::Url;
use serde_json::Value;
use zero2prod::in_memory::AppState;
use zero2prod::routes::SubscriptionRequest;
use common::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::common::get_id_from_response;
use crate::{common, subscriptions_confirm};

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let test_app = spawn_app().await;

    Mock::given(path("/v3/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.mock_email_server)
        .await;

    let response = test_app.post_subscriptions(
        &SubscriptionRequest::new("le guin".to_string(), 
        "ursula_le_guin@gmail.com".to_string())).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
    
    let response_body = response.text().await.unwrap();

    // Extract the "id" field
    let subscription_id = get_id_from_response(response_body);
    assert_eq!("1", subscription_id);

    assert!(!subscription_id.is_empty());
}
#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let test_app = spawn_app().await;
    let test_cases = vec![
        (SubscriptionRequest::new("le guin".to_string(), "".to_string()), "missing the email"),
        (SubscriptionRequest::new("".to_string(), "ursula_le_guin@gmail.com".to_string()), "missing the name"),
        (SubscriptionRequest::new("".to_string(), "".to_string()), "missing both name and email"),
        (SubscriptionRequest::new("Boo".to_string(), "my-gosh-not-an-email".to_string()), "invalid email"),
        (SubscriptionRequest::new("G".to_string(), "g@mail.com".to_string()), "too short of a name"),
        (SubscriptionRequest::new("the%estna^^e".to_string(), "mine@yahoo.com".to_string()), "invalid name"),
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        
        let response = test_app.post_subscriptions(&invalid_body).await;
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {

    let app = spawn_app().await;
    let request_body = SubscriptionRequest::new("le guin".to_string(), "ursula_le_guin@gmail.com".to_string());
    Mock::given(path("/v3/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.mock_email_server)
        .await;
    app.post_subscriptions(&request_body).await;

    // Mock asserts on drop
}

#[tokio::test]
async fn double_subscribe_does_not_create_a_new_subscription(){
    let app = spawn_app().await;
    let request_body = SubscriptionRequest::new("le guin".to_string(), "ursula_le_guin@gmail.com".to_string());
    Mock::given(path("/v3/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(2)
        .mount(&app.mock_email_server)
        .await;
    let response = app.post_subscriptions(&request_body).await;

    let response_body = response.text().await.unwrap();
    
    let first_subscription_id = get_id_from_response(response_body);
    
    // second request with same email
    let response = app.post_subscriptions(&request_body).await;
    let response_body = response.text().await.unwrap();
    
    let second_subscription_id = get_id_from_response(response_body);
    assert_eq!(first_subscription_id,second_subscription_id)

}

#[tokio::test]
async fn second_subscribe_after_confirmation_returns_bad_request(){
    let app = spawn_app().await;
    let request_body = SubscriptionRequest::new("le guin".to_string(), "ursula_le_guin@gmail.com".to_string());
    Mock::given(path("/v3/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.mock_email_server)
        .await;
    let response = app.post_subscriptions(&request_body).await;

    let response_body = response.text().await.unwrap();
    
    let first_subscription_id = get_id_from_response(response_body);
    
    let email_request = &app.mock_email_server.received_requests().await.unwrap()[0];
    let body: serde_json::Value = serde_json::from_slice(&email_request.body)
    .unwrap();
    // Extract the link from one of the request fields.
    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
        .links(s)
        .filter(|l| *l.kind() == linkify::LinkKind::Url)
        .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };
    let raw_confirmation_link = &get_link(&body["Html-part"].as_str().unwrap());
    let mut confirmation_link = Url::parse(raw_confirmation_link).unwrap();
    // Let's make sure we don't call random APIs on the web
    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
    confirmation_link.set_port(Some(app.port)).unwrap();
    // Act - we make the actual request to the confirm endpoint
    let response = reqwest::get(confirmation_link)
        .await
        .unwrap();
    // second request with same email
    let response = app.post_subscriptions(&request_body).await;
    assert_eq!(response.status().as_u16(), 400);
}