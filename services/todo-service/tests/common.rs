//! shared setup helpers for integration tests.

use diesel::r2d2::Pool;
use diesel_migrations::MigrationHarness;
use rust_toolbox::diesel_tools::{DatabaseManager, DatabasePool, DatabasePooledConnection};
use std::net::SocketAddr;
use todo_service::MIGRATIONS;

/// a single, migrated connection backed by an in-memory sqlite db.
pub fn connection() -> DatabasePooledConnection {
    let pool = Pool::builder()
        .max_size(1)
        .build(DatabaseManager::new(":memory:"))
        .expect("failed to build pool");
    let mut conn = pool.get().expect("failed to get connection");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("failed to run migrations");
    conn
}

/// a real, file-backed, migrated pool.
pub fn pool() -> DatabasePool {
    let db_path = format!(
        "/tmp/todo-service-test-{}-{}.db",
        std::process::id(),
        nanos_id()
    );
    let _ = std::fs::remove_file(&db_path);

    let pool: DatabasePool = Pool::builder()
        .build(DatabaseManager::new(db_path))
        .expect("failed to build pool");
    pool.get()
        .expect("failed to get connection")
        .run_pending_migrations(MIGRATIONS)
        .expect("failed to run migrations");
    pool
}

fn nanos_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

/// boots the todo service on a loopback address and returns it once ready.
pub async fn spawn_server(pool: DatabasePool) -> SocketAddr {
    use proto::todo_service::todo_service_client::TodoServiceClient;
    use rust_toolbox::diesel_tools::DatabaseService;
    use todo_service::service::TodoService;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind test listener");
    let addr = listener.local_addr().expect("failed to read local addr");

    let todo_service = TodoService::new(pool);

    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(todo_service.into_server())
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await
            .expect("server error");
    });

    loop {
        if TodoServiceClient::connect(format!("http://{addr}"))
            .await
            .is_ok()
        {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }

    addr
}
