use log::info;
use simple_logger::SimpleLogger;
use sqlx::PgPool;
use std::net::TcpListener;
use warcse::configuration::get_configuration;
use warcse::startup::run;

/// Application startup.
/// Binds to the given address.
///
/// # Returns
///
/// * Ok    - The application is up and running.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger.
    SimpleLogger::new().with_utc_timestamps().init().unwrap();

    // Fetch the configuration.
    let configuration = get_configuration().expect("Failed to read configuration");

    // Generate the connection pool.
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    // Create the address to bind to.
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(&address)?;

    info!("Binding to address: {}", address);
    run(listener, connection_pool)?.await?;
    Ok(())
}
