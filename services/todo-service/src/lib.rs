//! persistence and gRPC service layer for the todo-service binary; split
//! into a library so integration tests (in `tests/`) can exercise it
//! directly.

pub mod model;
mod proto;
pub mod repository;
pub mod schema;
pub mod service;

use diesel_migrations::{EmbeddedMigrations, embed_migrations};

/// this crate's diesel migrations, embedded into the binary at compile time
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// namespaces every `*ServiceError`'s
/// [`ServiceError::code`](rust_toolbox::tonic_tools::ServiceError::code) in
/// this crate, so a code coincidentally shared with another service can't
/// collide (see `google.rpc.ErrorInfo.domain`).
pub const SERVICE_ERROR_DOMAIN: &str = "todo-service.{{crate_name}}";
