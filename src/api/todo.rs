use leptos::prelude::*;

use crate::domain::todo::Todo;

#[server(CreateTodo, "/api")]
pub async fn create_todo_server(todo: Todo) -> Result<Todo, ServerFnError> {
    use crate::services::cosmos::todo_repository::get_cosmos_service;

    let cosmos_service = get_cosmos_service()
        .map_err(|e| ServerFnError::new(format!("Failed to get Cosmos service: {}", e)))?;

    cosmos_service
        .create_todo(todo)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create todo: {}", e)))
}

#[server(name=GetTodos, prefix="/api")]
pub async fn get_todos_server() -> Result<Vec<Todo>, ServerFnError> {
    use crate::services::cosmos::todo_repository::get_cosmos_service;

    let cosmos_service = get_cosmos_service()
        .map_err(|e| ServerFnError::new(format!("Failed to get Cosmos service: {}", e)))?;

    let cosmos_todos = cosmos_service
        .get_todos()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get todos: {}", e)))?;
    let todos: Vec<Todo> = cosmos_todos.into_iter().map(Todo::from).collect();

    Ok(todos)
}

#[server(UpdateTodo, "/api")]
pub async fn update_todo_server(todo: Todo) -> Result<Todo, ServerFnError> {
    use crate::services::cosmos::todo_repository::get_cosmos_service;

    let cosmos_service = get_cosmos_service()
        .map_err(|e| ServerFnError::new(format!("Failed to get Cosmos service: {}", e)))?;

    let cosmos_todo = cosmos_service
        .update_todo(todo)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update todo: {}", e)))?;

    Ok(Todo::from(cosmos_todo))
}

#[server(DeleteTodo, "/api")]
pub async fn delete_todo_server(todo_id: String) -> Result<(), ServerFnError> {
    use crate::services::cosmos::todo_repository::get_cosmos_service;

    let cosmos_service = get_cosmos_service()
        .map_err(|e| ServerFnError::new(format!("Failed to get Cosmos service: {}", e)))?;

    cosmos_service
        .delete_todo(&todo_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to delete todo: {}", e)))?;

    Ok(())
}
