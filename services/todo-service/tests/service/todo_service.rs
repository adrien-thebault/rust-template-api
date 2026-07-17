//! end-to-end gRPC tests for `{{crate_name}}.todo_service.TodoService`.

use crate::common;
use proto::todo_service::{
    CreateTodoRequest, DeleteTodoRequest, GetTodoRequest, ListTodosRequest, UpdateTodoRequest,
    todo_service_client::TodoServiceClient,
};

#[tokio::test]
async fn creates_updates_lists_and_deletes_a_todo() {
    let addr = common::spawn_server(common::pool()).await;
    let mut client = TodoServiceClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    let created = client
        .create_todo(CreateTodoRequest {
            title: "first".to_string(),
        })
        .await
        .expect("create todo")
        .into_inner();
    assert_eq!(created.title, "first");
    assert!(!created.done);

    let fetched = client
        .get_todo(GetTodoRequest { id: created.id })
        .await
        .expect("get todo")
        .into_inner();
    assert_eq!(fetched.id, created.id);

    let updated = client
        .update_todo(UpdateTodoRequest {
            id: created.id,
            title: created.title.clone(),
            done: true,
        })
        .await
        .expect("update todo")
        .into_inner();
    assert!(updated.done);

    let listed = client
        .list_todos(ListTodosRequest { page_request: None })
        .await
        .expect("list todos")
        .into_inner();
    assert_eq!(listed.page.unwrap().total_elements, 1);
    assert_eq!(listed.todos[0].id, created.id);

    client
        .delete_todo(DeleteTodoRequest { id: created.id })
        .await
        .expect("delete todo");

    let listed_after_delete = client
        .list_todos(ListTodosRequest { page_request: None })
        .await
        .expect("list todos")
        .into_inner();
    assert_eq!(listed_after_delete.page.unwrap().total_elements, 0);
}

#[tokio::test]
async fn getting_an_unknown_todo_returns_not_found() {
    let addr = common::spawn_server(common::pool()).await;
    let mut client = TodoServiceClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    let err = client
        .get_todo(GetTodoRequest { id: 404 })
        .await
        .expect_err("expected not-found");
    assert_eq!(err.code(), tonic::Code::NotFound);
}
