use crate::{
    model::{Todo, TodoError},
    repository::TodoRepository,
};
use proto::todo_service::{
    CreateTodoRequest, DeleteTodoRequest, GetTodoRequest, ListTodosRequest, ListTodosResponse,
    Todo as ProtoTodo, UpdateTodoRequest,
    todo_service_server::{TodoService as TodoServiceTrait, TodoServiceServer},
};
use rust_toolbox::diesel_tools::{
    DatabaseError, DatabasePool, DatabasePooledConnection, DatabaseService, EntityService,
};
use rust_toolbox::tonic_tools::ServiceError;
use std::collections::HashMap;
use thiserror::Error;
use tonic::{Code, Request, Response, Status, async_trait};

/// implements `{{crate_name}}.todo_service.TodoService`
#[derive(Clone)]
pub struct TodoService {
    pool: DatabasePool,
}

impl DatabaseService for TodoService {
    type Server = TodoServiceServer<Self>;

    fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    fn into_server(self) -> Self::Server {
        TodoServiceServer::new(self)
    }
}

impl TodoService {
    fn connection(&self) -> Result<DatabasePooledConnection, TodoServiceError> {
        Ok(self.pool.get().map_err(DatabaseError::from)?)
    }
}

impl EntityService<TodoRepository> for TodoService {
    type Error = TodoServiceError;
}

/// errors that can occur while serving todo-related gRPC requests
#[derive(Debug, Error)]
pub enum TodoServiceError {
    #[error("todo {0} not found")]
    NotFound(i32),
    #[error(transparent)]
    Todo(#[from] TodoError),
    #[error(transparent)]
    Database(#[from] DatabaseError),
}

impl ServiceError for TodoServiceError {
    fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "TODO_NOT_FOUND",
            Self::Todo(_) | Self::Database(_) => "TODO_INTERNAL_ERROR",
        }
    }

    fn domain(&self) -> &'static str {
        crate::SERVICE_ERROR_DOMAIN
    }

    fn status_code(&self) -> Code {
        match self {
            Self::NotFound(_) => Code::NotFound,
            Self::Todo(_) | Self::Database(_) => Code::Internal,
        }
    }

    fn metadata(&self) -> HashMap<String, String> {
        match self {
            Self::NotFound(_) => HashMap::new(),
            Self::Todo(_) | Self::Database(_) => {
                HashMap::from([("detail".to_string(), self.to_string())])
            }
        }
    }
}

impl From<TodoServiceError> for Status {
    fn from(err: TodoServiceError) -> Self {
        err.to_status()
    }
}

#[async_trait]
impl TodoServiceTrait for TodoService {
    async fn create_todo(
        &self,
        request: Request<CreateTodoRequest>,
    ) -> Result<Response<ProtoTodo>, Status> {
        let mut conn = self.connection()?;
        let req = request.into_inner();

        let entity = Todo {
            id: None,
            title: req.title,
            done: false,
            created_at: chrono::Utc::now().naive_utc(),
        };

        let saved = self.save(&mut conn, &entity)?;
        let proto: ProtoTodo = saved.try_into().map_err(TodoServiceError::from)?;
        Ok(Response::new(proto))
    }

    async fn get_todo(
        &self,
        request: Request<GetTodoRequest>,
    ) -> Result<Response<ProtoTodo>, Status> {
        let mut conn = self.connection()?;
        let id = request.into_inner().id as i32;

        let todo = self
            .find_by_id(&mut conn, &id)?
            .ok_or(TodoServiceError::NotFound(id))?;
        let proto: ProtoTodo = todo.try_into().map_err(TodoServiceError::from)?;
        Ok(Response::new(proto))
    }

    async fn update_todo(
        &self,
        request: Request<UpdateTodoRequest>,
    ) -> Result<Response<ProtoTodo>, Status> {
        let mut conn = self.connection()?;
        let req = request.into_inner();
        let id = req.id as i32;

        // preserve `created_at`: the update request only carries the
        // user-editable fields, not every column
        let existing = self
            .find_by_id(&mut conn, &id)?
            .ok_or(TodoServiceError::NotFound(id))?;

        let entity = Todo {
            id: Some(id),
            title: req.title,
            done: req.done,
            created_at: existing.created_at,
        };

        let saved = self.save(&mut conn, &entity)?;
        let proto: ProtoTodo = saved.try_into().map_err(TodoServiceError::from)?;
        Ok(Response::new(proto))
    }

    async fn delete_todo(
        &self,
        request: Request<DeleteTodoRequest>,
    ) -> Result<Response<()>, Status> {
        let mut conn = self.connection()?;
        let id = request.into_inner().id as i32;

        self.delete_by_id(&mut conn, &id)?;
        Ok(Response::new(()))
    }

    async fn list_todos(
        &self,
        request: Request<ListTodosRequest>,
    ) -> Result<Response<ListTodosResponse>, Status> {
        let mut conn = self.connection()?;
        let page_request = request
            .into_inner()
            .page_request
            .map(Into::into)
            .unwrap_or_default();

        let page = self.find_all(&mut conn, page_request)?;
        let page_response = proto::shared::PageResponse::from(&page);

        let todos = page
            .data
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<ProtoTodo>, TodoError>>()
            .map_err(TodoServiceError::from)?;

        Ok(Response::new(ListTodosResponse {
            todos,
            page: Some(page_response),
        }))
    }
}
