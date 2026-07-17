//! integration test entry point. Submodules mirror `src/`'s layout; see
//! `common.rs` for the shared setup helpers they build on.

mod common;

#[path = "repository/todo_repository.rs"]
mod todo_repository;

#[path = "service/todo_service.rs"]
mod todo_service;
