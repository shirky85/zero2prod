use wiremock::matchers::{any, method, path};
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
    
    app.create_unconfirmed_subscription().await;

    let _mock_send_newsletter = Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.mock_email_server)
        .await;

    // Act
    let response = app.post_newsletters(body).await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
    // Mock verifies on Drop that we haven't sent the newsletter email
}

#[tokio::test]
async fn newletter_is_sent_to_confirmed_subscribers(){
    // Arrange
    let app = spawn_app().await;
    let body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>",
        }
    });
    
    app.create_confirmed_subscription().await;

    let _mock_send_newsletter = Mock::given(path("/v3/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.mock_email_server)
        .await;

    let response = app.post_newsletters(body).await;
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    // Mock verifies on Drop that we haven't sent the newsletter email
}

#[tokio::test]
async fn newsletters_returns_a_400_for_invalid_data(){
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        (serde_json::json!({
            "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>",
            }
        }), "missing the title"),

        (serde_json::json!({
            "title": "some title"
        }), "missing the content"),

        (serde_json::json!({
            "title": "some title",
            "content": {
                "html": "<p>Newsletter body as HTML</p>",
            }
        }), "missing the text in content"),

        (serde_json::json!({
            "title": "some title",
            "content": {
                "text": "short text",
            }
        }), "missing the html in content"),

        (serde_json::json!({
            "title": "this is a very long title, really too long, what are you doing, are you crazy?",
            "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>",
            }
        }), "title is too long"),

        (serde_json::json!({
            "title": "short title",
            "content": {
                "text": "bla",
                "html": "<p>Newsletter body as HTML</p>",
            }
        }), "content is too short"),
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        
        let response = app.post_newsletters(invalid_body).await;
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
async fn newsletters_returns_a_502_on_send_email_error(){
    // Arrange
    let app = spawn_app().await;
    let body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>",
        }
    });
    
    app.create_confirmed_subscription().await;

    let _mock_send_newsletter = Mock::given(path("/v3/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .expect(1)
        .mount(&app.mock_email_server)
        .await;

    let response = app.post_newsletters(body).await;
    // Assert
    assert_eq!(response.status().as_u16(), 502);
}
