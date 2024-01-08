use crate::routes::{query, stats, status};
use actix_cors::Cors;
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;

/// Run the application in the given port.
///
/// # Arguments
///
/// * listener  - `TcpListener` to bind to.
/// * db_pool   - `PgPool` the PostgreSQL pool.
///
/// # Returns
///
/// * `Server` if the bind was successful.
/// * `std::io::Error` if the app did not start successfully.
pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .service(actix_files::Files::new("/static/", "ui/dist").index_file("index.html"))
            .route("/status", web::get().to(status))
            .route("/query", web::post().to(query))
            .route("/stats", web::get().to(stats))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
