use wiremock::matchers::any;
use wiremock::{Mock, ResponseTemplate};

use crate::common::spawn_app;

#[tokio::test]
async fn newsletters_are_not_sent_to_unconfirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    let body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>",
        }
    });
    
    app.create_uncorfirmed_subscription().await;

    let _mock_send_newsletter = Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.mock_email_server)
        .await;

    // Act
    let response = reqwest::Client::new()
        .post(&format!("{}/newsletters", app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 200);
    // Mock verifies on Drop that we haven't sent the newsletter email
}