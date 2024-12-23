use zero2prod::routes::SubscriptionRequest;
mod common;
use common::spawn_app;
use wiremock::matchers::{any, method};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let test_app = spawn_app().await;
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
    // Arrange
    let test_app = spawn_app().await;
    
    Mock::given(any())
    .and(method("POST"))
    .respond_with(ResponseTemplate::new(200))
    .expect(1)
    .mount(&test_app.mock_email_server)
    .await;

    // Act
    let _response = test_app.post_subscriptions(SubscriptionRequest::new("le guin".to_string(), "ursula_le_guin@gmail.com".to_string()));

  
    // Assert
    // Mock asserts on drop
}