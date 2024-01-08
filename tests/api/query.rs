use crate::app::spawn_app;

#[actix_rt::test]
async fn test_query_404() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let data = "{\"query\":\"test\"}";

    // Act
    let response = client
        .post(&format!("{}/imaginaryword", &app.address))
        .header("Content-Type", "application/json")
        .body(data)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 404);
}
