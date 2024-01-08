use crate::services::get_stats;
use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::PgPool;

/// Check if the application is up and running.
///
/// # Arguments
///
/// * _req  - `HttpRequest` the HTTP Request.
///
/// # Returns
///
/// * `HttpResponse`, 200 OK if the app is up and running.
pub async fn status(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().finish()
}

/// Get the status of the database for the application.
///
/// # Arguments
///
/// * _req  - `HttpRequest` the HTTP Request.
///
/// # Returns
pub async fn stats(
    _req: HttpRequest,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    // Perform the search.
    let result = get_stats(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Convert to JSON.
    let result = serde_json::to_string_pretty(&result)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(result))
}
