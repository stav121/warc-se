use sqlx::PgPool;
use std::net::TcpListener;
use warcse::configuration::{get_configuration, DatabaseSettings};
use warcse::startup::run;

/// A test application.
///
/// Contains the address to bind to and the connection pool.
pub(crate) struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

/// Spawn an application instance.
pub(crate) async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = configure_database(&configuration.database).await;

    let server = run(listener, connection_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

/// Create a database configuration.
pub(crate) async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Migrate database
    PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.")
}
