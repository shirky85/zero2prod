use reqwest::Url;
use wiremock::{ResponseTemplate, Mock};
use wiremock::matchers::{path, method};
use zero2prod::routes::SubscriptionRequest;

use crate::common::{get_id_from_response, spawn_app};

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    // Arrange
    let app = spawn_app().await;
    let request_body = SubscriptionRequest::new("le guin".to_string(), "ursula_le_guin@gmail.com".to_string());
    Mock::given(path("/v3/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.mock_email_server)
        .await;
    
    let response = app.post_subscriptions(&request_body).await;
    // Assert
    let email_request = &app.mock_email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);
    // The two links should be identical
    assert_eq!(confirmation_links.html, confirmation_links.plain_text);
    assert_eq!(response.status().as_u16(), 200);
    let response_body = response.text().await.unwrap();
    
    let subscription_id = get_id_from_response(response_body);
    // Act
    let raw_confirmation_link = confirmation_links.html.as_str();
    let mut confirmation_link = Url::parse(raw_confirmation_link).unwrap();
    // Let's make sure we don't call random APIs on the web
    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
    confirmation_link.set_port(Some(app.port)).unwrap();
    // Act - we make the actual request to the confirm endpoint
    let _response = reqwest::get(confirmation_link)
        .await
        .unwrap();
    // Assert
    

    let subscriptions_data = app.get_subscription(&subscription_id).await.text().await.unwrap();
    let subscriptions_data_json: serde_json::Value = serde_json::from_str(&subscriptions_data).unwrap();
    let subscription_status = subscriptions_data_json["status"].as_str().unwrap();
    assert_eq!(subscription_status, "confirmed");
}


#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    // Arrange
    let app = spawn_app().await;
    // Act
    let response = reqwest::get(&format!("{}/subscriptions/confirm", app.address))
        .await
        .unwrap();
    // Assert
    assert_eq!(response.status().as_u16(), 400);
}
