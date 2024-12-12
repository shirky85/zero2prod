use std::net::TcpListener;


#[tokio::test]
async fn health_check_works() {
    // Since I don't want the tests to run on a fixed port I randomize it by using port 0, 
    // and later extracting the true port to use it in the client request.
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed binding to port");
    let address = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port().to_string());
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
    assert_eq!("\"{\\\"metric1\\\":1000, \\\"metric2\\\":2000}\"", response.text().await.expect("The body is not as expected or empty"));
}
// Launch our application in the background ~somehow~ 
fn spawn_app(listener : TcpListener)  {
    let server = zero2prod::run(listener).expect("Failed to bind address");
    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it here, hence the non-binding let
    let _ = tokio::spawn(server);
}