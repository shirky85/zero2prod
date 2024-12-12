use std::net::TcpListener;
use actix_web::web;
use zero2prod::SubscriptionRequest;



#[tokio::test]
async fn health_check_works() {
    // Since I don't want the tests to run on a fixed port I randomize it by using port 0,
    // and later extracting the true port to use it in the client request.
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed binding to port");
    let address = format!(
        "http://127.0.0.1:{}",
        listener.local_addr().unwrap().port().to_string()
    );
    // Arrange
    spawn_app(listener);
    // We need to bring in `reqwest`
    // to perform HTTP requests against our application.
    let client = reqwest::Client::new();
    // Act
    let response = client
        .get(&format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(
        "\"{\\\"metric1\\\":1000, \\\"metric2\\\":2000}\"",
        response
            .text()
            .await
            .expect("The body is not as expected or empty")
    );
}
// Launch our application in the background ~somehow~
fn spawn_app(listener: TcpListener) {
    let server = zero2prod::run(listener).expect("Failed to bind address");
    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it here, hence the non-binding let
    let _ = tokio::spawn(server);
}

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
