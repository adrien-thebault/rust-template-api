use crate::model::Todo;
use rust_toolbox::impl_repository;

/// data access for [`Todo`]s. Ids are database-assigned: save a todo
/// with `id: None` to create it, `id: Some(_)` to update it. `find_all`
/// (from [`rust_toolbox::diesel_tools::EntityService`]) can sort on `id`,
/// `title` or `created_at` - add more columns to `SortColumns` as your own
/// entity grows fields worth sorting on.
pub struct TodoRepository;

impl_repository!(TodoRepository {
    Schema = crate::schema::todos,
    Entity = Todo,
    Id = (i32, id, autoincrement),
    SortColumns = { id, title, created_at },
});
