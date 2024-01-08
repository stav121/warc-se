use sqlx::PgPool;

/// General information for the application.
#[derive(Debug, serde::Serialize)]
pub struct StatsResponse {
    pub corpus_count: Option<i64>,
    pub record_count: Option<i64>,
    pub word_count: Option<i64>,
}

/// Get the general stats of the application.
///
/// # Arguments
///
/// * pool  - `PgPool` the PostgreSQL pool.
///
/// # Returns
///
/// `Result<StatsResponse, sqlx::Error>` depending on the response from PostgreSQL.
pub async fn get_stats(pool: &PgPool) -> Result<StatsResponse, sqlx::Error> {
    let status: StatsResponse = sqlx::query_as!(
        StatsResponse,
        r#"
            SELECT (SELECT COUNT(*) FROM corpus_info)  AS corpus_count,
                   (SELECT COUNT(*) FROM record_index) AS record_count,
                   (SELECT COUNT(*) FROM word_index) AS word_count
        "#,
    )
    .fetch_one(pool)
    .await?;
    Ok(status)
}
