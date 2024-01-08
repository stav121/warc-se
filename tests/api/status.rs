use crate::app::spawn_app;

/// Perform a request on /status endpoint.
/// Should return 200 OK.
#[actix_rt::test]
async fn test_status_200() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/status", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
