use clap::Parser;
use diesel_migrations::MigrationHarness;
use dotenvy::dotenv;
use rust_toolbox::diesel_tools::{DatabaseError, DatabaseManager, DatabasePool, DatabaseService};
use rust_toolbox::tower_tools::layers::{
    grpc_trace_layer, propagate_request_id_layer, request_id_context_layer, request_id_layer,
};
use todo_service::MIGRATIONS;
use todo_service::service::TodoService;
use tonic::transport::Server;
use tracing::Level;

#[derive(Parser)]
struct Args {
    /// path to the database file (SQLite)
    #[arg(long, env = "DATABASE_URL", default_value = "todo-service.db")]
    database_url: String,

    /// address to listen on for gRPC requests
    #[arg(long, env = "LISTEN_ADDR", default_value = "0.0.0.0:50051")]
    listen_addr: String,

    /// increase verbosity (can be repeated, e.g. -vv)
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    verbose: u8,

    /// decrease verbosity (can be repeated)
    #[arg(short = 'q', long = "quiet", action = clap::ArgAction::Count)]
    quiet: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenv();

    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_max_level(match args.verbose as i16 - args.quiet as i16 {
            i16::MIN..=-2 => Level::ERROR,
            -1 => Level::WARN,
            0 => Level::INFO,
            1 => Level::DEBUG,
            2..=i16::MAX => Level::TRACE,
        })
        .init();

    let pool = DatabasePool::builder().build(DatabaseManager::new(args.database_url))?;

    pool.get()?
        .run_pending_migrations(MIGRATIONS)
        .map_err(DatabaseError::Migration)?;

    let todo_service = TodoService::new(pool);

    Server::builder()
        // first `.layer()` is outermost under tonic/tower::ServiceBuilder
        // composition: assign a request id first, expose it as ambient
        // context, then propagate/trace using it.
        .layer(request_id_layer())
        .layer(request_id_context_layer())
        .layer(propagate_request_id_layer())
        .layer(grpc_trace_layer())
        .add_service(todo_service.into_server())
        .serve(args.listen_addr.parse()?)
        .await?;

    Ok(())
}
