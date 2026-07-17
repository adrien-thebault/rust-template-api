use diesel::prelude::*;
use thiserror::Error;

/// example CRUD entity - rename/reshape this to fit your own domain; see the
/// module doc comment in `proto/src/todo-service.proto` for why a todo list.
///
/// `id` is `Option<i32>`: `None` lets the database assign it on insert
/// (`AUTOINCREMENT`), `Some` upserts by that id - see
/// [`crate::repository::TodoRepository`]'s `impl_repository!` call.
#[derive(
    Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable, Insertable, AsChangeset,
)]
#[diesel(table_name = crate::schema::todos)]
#[diesel(primary_key(id))]
pub struct Todo {
    #[diesel(deserialize_as = i32)]
    pub id: Option<i32>,
    pub title: String,
    pub done: bool,
    pub created_at: chrono::NaiveDateTime,
}

/// errors related to converting a [`Todo`] to/from its wire representation
#[derive(Debug, Error)]
pub enum TodoError {
    /// attempted to convert a todo that hasn't been persisted (and so has
    /// no database-assigned id) into its wire representation
    #[error("todo has not been persisted yet")]
    MissingId,
}
