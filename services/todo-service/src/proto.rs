//! conversions between domain/entity types (see [`crate::model`]) and their
//! gRPC wire representation (`proto::todo_service`).

use crate::model::{Todo, TodoError};

impl TryFrom<Todo> for proto::todo_service::Todo {
    type Error = TodoError;

    fn try_from(todo: Todo) -> Result<Self, Self::Error> {
        Ok(Self {
            id: todo.id.ok_or(TodoError::MissingId)? as u32,
            title: todo.title,
            done: todo.done,
            created_at: todo.created_at.and_utc().to_rfc3339(),
        })
    }
}
