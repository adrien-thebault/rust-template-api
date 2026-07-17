//! REST routes, one module per resource. Each module owns a
//! [`rust_toolbox::axum_tools::controller::Controller`] that builds the
//! `Router` for the routes it's responsible for; this file only composes
//! them together. Handlers translate between JSON and the matching gRPC
//! request/response, delegating auth/RBAC to `rust_toolbox::axum_tools::auth`
//! and error mapping to `rust_toolbox::axum_tools::api_error::ApiError`.

pub mod auth;
pub mod todos;

use crate::AppState;
use axum::Router;
use rust_toolbox::axum_tools::controller::Controller;

pub fn router(state: AppState) -> Router {
    Router::new()
        .nest(
            "/api",
            Router::new()
                .merge(auth::AuthController.router())
                .merge(todos::TodosController.router()),
        )
        .with_state(state)
}
