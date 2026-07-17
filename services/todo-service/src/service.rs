//! gRPC service implementation, backed by a
//! [`rust_toolbox::diesel_tools::DatabasePool`].

mod todo_service;

pub use todo_service::*;
