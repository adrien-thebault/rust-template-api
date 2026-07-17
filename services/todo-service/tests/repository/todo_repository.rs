//! integration tests for `todo_service::repository::TodoRepository`.

use crate::common;
use rust_toolbox::diesel_tools::{Find, PageRequest, Save};
use todo_service::model::Todo;
use todo_service::repository::TodoRepository;

fn new_todo(title: &str) -> Todo {
    Todo {
        id: None,
        title: title.to_string(),
        done: false,
        created_at: chrono::Utc::now().naive_utc(),
    }
}

#[test]
fn creates_a_new_todo_with_a_database_assigned_id() {
    let mut conn = common::connection();

    let created = TodoRepository::save(&mut conn, &new_todo("first")).expect("create todo");
    assert_eq!(created.id, Some(1));
    assert_eq!(created.title, "first");
}

#[test]
fn updates_an_existing_todo_in_place() {
    let mut conn = common::connection();

    let created = TodoRepository::save(&mut conn, &new_todo("first")).expect("create todo");
    let updated = TodoRepository::save(
        &mut conn,
        &Todo {
            done: true,
            ..created.clone()
        },
    )
    .expect("update todo");

    assert_eq!(updated.id, created.id);
    assert!(updated.done);
}

#[test]
fn find_all_paginates_and_counts_every_todo() {
    let mut conn = common::connection();

    TodoRepository::save(&mut conn, &new_todo("first")).expect("create first todo");
    TodoRepository::save(&mut conn, &new_todo("second")).expect("create second todo");

    let page = TodoRepository::find_all(&mut conn, PageRequest::default()).expect("find all todos");
    assert_eq!(page.total_elements, 2);
    assert_eq!(page.data.len(), 2);
}
