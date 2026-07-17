use crate::AppState;
use crate::auth::Role;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post, put},
};
use proto::shared::PageRequest;
use proto::todo_service::{
    CreateTodoRequest, DeleteTodoRequest, GetTodoRequest, ListTodosRequest, ListTodosResponse,
    Todo, UpdateTodoRequest,
};
use rust_toolbox::axum_tools::{api_error::ApiError, auth::User, controller::Controller};
use serde_derive::Deserialize;

/// query params for [`TodosController::list_todos`]
#[derive(Deserialize)]
pub struct ListTodosQuery {
    offset: Option<i64>,
    size: Option<i64>,
}

/// owns `/api/todos*` and `/api/admin/todos*`
pub struct TodosController;

impl Controller<AppState> for TodosController {
    fn router(&self) -> Router<AppState> {
        Router::new()
            .nest(
                "/todos",
                Router::new()
                    .route("/", get(Self::list_todos))
                    .route("/{id}", get(Self::get_todo)),
            )
            .nest(
                "/admin/todos",
                Router::new()
                    .route("/", post(Self::create_todo))
                    .route("/{id}", put(Self::update_todo).delete(Self::delete_todo)),
            )
    }
}

impl TodosController {
    /// `GET /api/todos` - public
    async fn list_todos(
        State(mut state): State<AppState>,
        Query(query): Query<ListTodosQuery>,
    ) -> Result<Json<ListTodosResponse>, ApiError> {
        let page_request = match (query.offset, query.size) {
            (Some(offset), Some(size)) => Some(PageRequest {
                offset: Some(offset),
                size: Some(size),
                sort: vec![],
            }),
            _ => None,
        };

        let todos = state
            .todos
            .list_todos(ListTodosRequest { page_request })
            .await?
            .into_inner();
        Ok(Json(todos))
    }

    /// `GET /api/todos/{id}` - public
    async fn get_todo(
        State(mut state): State<AppState>,
        Path(id): Path<u32>,
    ) -> Result<Json<Todo>, ApiError> {
        let todo = state
            .todos
            .get_todo(GetTodoRequest { id })
            .await?
            .into_inner();
        Ok(Json(todo))
    }

    /// `POST /api/admin/todos` - requires [`Role::Admin`]
    async fn create_todo(
        State(mut state): State<AppState>,
        user: User,
        Json(req): Json<CreateTodoRequest>,
    ) -> Result<Json<Todo>, ApiError> {
        user.require_role(Role::Admin)?;
        let todo = state.todos.create_todo(req).await?.into_inner();
        Ok(Json(todo))
    }

    /// `PUT /api/admin/todos/{id}` - requires [`Role::Admin`]
    async fn update_todo(
        State(mut state): State<AppState>,
        user: User,
        Path(id): Path<u32>,
        Json(mut req): Json<UpdateTodoRequest>,
    ) -> Result<Json<Todo>, ApiError> {
        user.require_role(Role::Admin)?;
        req.id = id;

        let todo = state.todos.update_todo(req).await?.into_inner();
        Ok(Json(todo))
    }

    /// `DELETE /api/admin/todos/{id}` - requires [`Role::Admin`]
    async fn delete_todo(
        State(mut state): State<AppState>,
        user: User,
        Path(id): Path<u32>,
    ) -> Result<(), ApiError> {
        user.require_role(Role::Admin)?;
        state.todos.delete_todo(DeleteTodoRequest { id }).await?;
        Ok(())
    }
}
