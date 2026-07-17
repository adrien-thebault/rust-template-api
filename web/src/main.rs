mod auth;
mod routes;

use auth::Role;
use clap::Parser;
use dotenvy::dotenv;
use proto::todo_service::todo_service_client::TodoServiceClient;
use rust_toolbox::axum_tools::auth::{
    AuthBackend, DecodingKey, EncodingKey, InMemoryBasicAuthBackend, JwtCodec, JwtCodecProvider,
};
use rust_toolbox::tonic_tools::{RequestIdChannel, request_id_interceptor};
use rust_toolbox::tower_tools::layers::{
    http_trace_layer, propagate_request_id_layer, request_id_context_layer, request_id_layer,
};
use std::{collections::HashMap, sync::Arc};
use tonic::transport::Endpoint;
use tracing::Level;

/// shared state for every route handler: gRPC clients to the backend
/// services (each wrapped once with [`request_id_interceptor`] so every
/// call carries the current request's id onward - see
/// [`request_id_context_layer`] - rather than re-built per handler), the
/// configured auth backend, and the JWT codec used to issue/verify admin
/// sessions.
#[derive(Clone)]
pub struct AppState {
    pub todos: TodoServiceClient<RequestIdChannel>,
    pub auth: Arc<dyn AuthBackend>,
    pub jwt: Arc<JwtCodec>,
}

impl JwtCodecProvider for AppState {
    fn jwt_codec(&self) -> &JwtCodec {
        &self.jwt
    }
}

#[derive(Parser)]
struct Args {
    /// address to listen on for HTTP requests
    #[arg(long, env = "WEB_LISTEN_ADDR", default_value = "0.0.0.0:8080")]
    listen_addr: String,

    /// address of todo-service
    #[arg(
        long,
        env = "TODO_SERVICE_ADDR",
        default_value = "http://127.0.0.1:50051"
    )]
    todo_service_addr: String,

    /// secret used to sign/verify admin sessions
    #[arg(long, env = "WEB_SESSION_SECRET")]
    session_secret: String,

    /// the sole in-memory admin account's username
    #[arg(long, env = "WEB_ADMIN_USERNAME", default_value = "admin")]
    admin_username: String,

    /// the sole in-memory admin account's password, as a sha256 hex digest
    /// (e.g. `printf '%s' 'the-password' | sha256sum`)
    #[arg(long, env = "WEB_ADMIN_PASSWORD_SHA256")]
    admin_password_sha256: String,

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

    let mut users = HashMap::new();
    users.insert(
        args.admin_username,
        (
            args.admin_password_sha256,
            vec![Role::Admin.as_ref().to_string()],
        ),
    );

    let todo_service = Endpoint::from_shared(args.todo_service_addr)?.connect_lazy();

    let state = AppState {
        todos: TodoServiceClient::with_interceptor(todo_service, request_id_interceptor),
        auth: Arc::new(InMemoryBasicAuthBackend::new(users)),
        jwt: Arc::new(JwtCodec::new(
            EncodingKey::from_secret(args.session_secret.as_bytes()),
            DecodingKey::from_secret(args.session_secret.as_bytes()),
        )),
    };

    // axum's `Router::layer` composes the opposite way tonic/tower's
    // `ServiceBuilder` does: the *last* `.layer()` call is outermost, so it
    // has to be request-id assignment here for it to run before the layers
    // that read it (propagation, tracing, context) - reverse of the order in
    // todo-service's `main.rs`.
    let app = routes::router(state)
        .layer(http_trace_layer())
        .layer(propagate_request_id_layer())
        .layer(request_id_context_layer())
        .layer(request_id_layer());

    let listener = tokio::net::TcpListener::bind(&args.listen_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
