use crate::domain::ResponseContainer;
use crate::services::perform_search;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use std::time::Instant;

/// Normal query.
#[derive(serde::Deserialize)]
pub struct FormData {
    query: String,
}

/// Perform a query
///
/// # Arguments
///
/// * form  - `web::Json` the input data. Contains the query.
/// * pool  - `PgPool` the PostgreSQL pool.
pub async fn query(
    form: web::Json<FormData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    // Calculate execution time.
    let start = Instant::now();

    // Perform the search.
    let result = perform_search(pool.get_ref(), &form.query)
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(e))?;

    // Get the execution time.
    let duration = start.elapsed();

    // Convert to JSON.
    let result = serde_json::to_string_pretty(&ResponseContainer {
        result_count: result.len(),
        result,
        duration: duration.as_millis(),
    })
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(result))
}
