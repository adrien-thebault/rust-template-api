//! domain/entity types: what's stored in the database and how it converts
//! to/from the gRPC wire format (see `src/proto.rs`).

mod todo;

pub use todo::*;
