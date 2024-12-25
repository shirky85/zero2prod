use zero2prod::routes::SubscriptionRequest;
use common::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::common;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let test_app = spawn_app().await;

    Mock::given(path("/v3/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.mock_email_server)
        .await;

    let response = test_app.post_subscriptions(
        SubscriptionRequest::new("le guin".to_string(), 
        "ursula_le_guin@gmail.com".to_string())).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
    assert_eq!(1.to_string(), response.text().await.unwrap())
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
        
        let response = test_app.post_subscriptions(invalid_body).await;
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
    app.post_subscriptions(request_body).await;

    // Mock asserts on drop
}

