//! data access layer: one repository per entity, built on `rust_toolbox`'s
//! `Repository`/`Find`/`Save`/`Delete` traits.

mod todo_repository;

pub use todo_repository::*;
