use std::net::TcpListener;
use actix_web::web;
use zero2prod::routes::SubscriptionRequest;
mod common;
use common::spawn_app;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed binding to port");
    let app_address = format!(
        "http://127.0.0.1:{}",
        listener.local_addr().unwrap().port().to_string()
    );
    spawn_app(listener);
    let client = reqwest::Client::new();
    // Act
    let request = web::Json(
        SubscriptionRequest::new("le guin".to_string(), "ursula_le_guin@gmail.com".to_string()));
    let json_payload = serde_json::to_string(&request).unwrap();
    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/json")
        .body(json_payload)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(200, response.status().as_u16());
    assert_eq!(1.to_string(), response.text().await.unwrap())
}
#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed binding to port");
    let app_address = format!(
        "http://127.0.0.1:{}",
        listener.local_addr().unwrap().port().to_string()
    );
    spawn_app(listener);
    let client = reqwest::Client::new();
    let test_cases = vec![
        (SubscriptionRequest::new("le guin".to_string(), "".to_string()), "missing the email"),
        (SubscriptionRequest::new("".to_string(), "ursula_le_guin@gmail.com".to_string()), "missing the name"),
        (SubscriptionRequest::new("".to_string(), "".to_string()), "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let request = web::Json(invalid_body);
        let response = client
            .post(&format!("{}/subscriptions", &app_address))
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&request).unwrap())
            .send()
            .await
            .expect("Failed to execute request.");
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
