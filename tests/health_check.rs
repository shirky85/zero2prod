mod common;
use common::spawn_app;


#[tokio::test]
async fn health_check_works() {
    let test_app = spawn_app().await;
    // We need to bring in `reqwest`
    // to perform HTTP requests against our application.
    let client = reqwest::Client::new();
    // Act
    let response = client
        .get(&format!("{}/health_check", test_app.address))
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


