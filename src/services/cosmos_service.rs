use azure_core::{credentials::Secret, error::Error as AzureError};
use azure_data_cosmos::{CosmosClient, PartitionKey};
use futures::stream::TryStreamExt;
use leptos::leptos_dom::logging;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::{
    config::get_config,
    todo::{Todo, TodoAssignee, TodoStatus},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosDbTodo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<i64>,
    pub assignee: String,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub partition_key: String,
    pub email: String,
}

impl CosmosDbTodo {
    /// Converts a `Todo` into a `CosmosDbTodo` for database storage.
    ///
    /// # Errors
    ///
    /// Returns an error if the app configuration cannot be retrieved or if the
    /// assignee email is not found in the configuration.
    pub fn try_from_todo(todo: Todo) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let now = chrono::Utc::now().timestamp();
        let config = get_config().map_err(|e| format!("Failed to get app config: {e}"))?;
        let email = config
            .emails
            .get(&todo.assignee)
            .ok_or("Assignee email not found")?;

        Ok(Self {
            id: todo.id,
            title: todo.title,
            description: todo.description,
            due_date: todo.due_date,
            assignee: todo.assignee.as_str().to_string(),
            status: todo.status.as_str().to_string(),
            created_at: now,
            updated_at: now,
            partition_key: "family_todos".to_string(),
            email: email.clone(),
        })
    }
}

impl From<CosmosDbTodo> for Todo {
    fn from(cosmos_todo: CosmosDbTodo) -> Self {
        Self {
            id: cosmos_todo.id.parse().unwrap_or(String::new()), // Convert string ID back to usize for UI
            title: cosmos_todo.title,
            description: cosmos_todo.description,
            due_date: cosmos_todo.due_date,
            assignee: TodoAssignee::from_str(&cosmos_todo.assignee).unwrap_or(TodoAssignee::Mikko),
            status: TodoStatus::from_str(&cosmos_todo.status).unwrap_or(TodoStatus::NotStarted),
        }
    }
}

pub struct CosmosService {
    client: CosmosClient,
    database_name: String,
    container_name: String,
}

impl CosmosService {
    /// Creates a new instance of the Cosmos DB service.
    ///
    /// # Errors
    ///
    /// Returns an error if the app configuration cannot be retrieved or if the Cosmos client
    /// cannot be initialized with the provided connection details.
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let app_config = get_config().map_err(|e| format!("Failed to get app config: {e}"))?;

        let client = CosmosClient::with_key(
            &app_config.cosmos.uri,
            Secret::from(app_config.cosmos.connection_string.as_str()),
            None,
        )?;

        Ok(Self {
            client,
            database_name: "familyleppanen".to_string(),
            container_name: "todos".to_string(),
        })
    }

    /// Creates a new todo item in the Cosmos DB container.
    ///
    /// # Errors
    ///
    /// Returns an `AzureError` if the creation operation fails or if there's an issue
    /// connecting to the Cosmos DB service.
    pub async fn create_todo(
        &self,
        todo: Todo,
    ) -> Result<Todo, Box<dyn std::error::Error + Send + Sync>> {
        let todo_cloned = todo.clone();
        let cosmos_todo = CosmosDbTodo::try_from_todo(todo)?;

        let database = self.client.database_client(&self.database_name);
        let container = database.container_client(&self.container_name);
        let partition_key = PartitionKey::from("family_todos");
        match container
            .create_item(partition_key, cosmos_todo, None)
            .await
        {
            Ok(_) => {
                logging::console_log(&format!("Created todo in Cosmos DB: {todo_cloned:#?}",));
                Ok(todo_cloned)
            }
            Err(e) => {
                logging::console_error("ERROR");
                eprintln!("Error creating todo in Cosmos DB: {e}");
                Err(Box::new(e))
            }
        }
    }

    /// Retrieves a list of todo items from the Cosmos DB container for a specific todo ID.
    ///
    /// # Errors
    ///
    /// Returns an `AzureError` if the query operation fails or if there's an issue
    /// connecting to the Cosmos DB service.
    pub async fn get_todos(&self) -> Result<Vec<CosmosDbTodo>, AzureError> {
        let database = self.client.database_client(&self.database_name);
        let container = database.container_client(&self.container_name);

        let query = "SELECT * FROM c ORDER BY c.created_at DESC";
        let partition_key = PartitionKey::from("family_todos");

        let query_result = container.query_items::<CosmosDbTodo>(query, partition_key, None);

        let mut todos = Vec::new();

        match query_result {
            Ok(mut query_stream) => {
                while let Ok(Some(feed_page)) = query_stream.try_next().await {
                    for item in feed_page.items() {
                        todos.push(item.clone());
                    }
                }
            }
            Err(e) => {
                eprintln!("Error querying todos: {e}");
                return Err(e);
            }
        }
        Ok(todos)
    }

    /// Updates a todo item in the Cosmos DB container
    ///
    /// # Errors
    ///
    /// Returns an `AzureError` if the update operation fails or if there's an issue
    /// connecting to the Cosmos DB service.
    pub async fn update_todo(
        &self,
        updated_todo: Todo,
    ) -> Result<CosmosDbTodo, Box<dyn std::error::Error + Send + Sync>> {
        let mut cosmos_todo = CosmosDbTodo::try_from_todo(updated_todo)?;
        cosmos_todo.updated_at = chrono::Utc::now().timestamp();
        let partition_key = PartitionKey::from("family_todos");

        let database = self.client.database_client(&self.database_name);
        let container = database.container_client(&self.container_name);

        let response = container
            .replace_item(partition_key, &cosmos_todo.id, &cosmos_todo, None)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        if !response.status().is_success() {
            let error_msg = format!("Failed to update todo in Cosmos DB: {}", response.status());
            logging::console_error(&error_msg);
            return Err(Box::new(std::io::Error::other(error_msg)));
        }
        Ok(cosmos_todo)
    }

    /// Deletes a todo item from the Cosmos DB container
    ///
    /// # Errors
    ///
    /// Returns an `AzureError` if the deletion operation fails or if there's an issue
    /// connecting to the Cosmos DB service.
    pub async fn delete_todo(&self, todo_id: &str) -> Result<(), AzureError> {
        let database = self.client.database_client(&self.database_name);
        let container = database.container_client(&self.container_name);
        let partition_key = PartitionKey::from("family_todos");

        container.delete_item(partition_key, todo_id, None).await?;

        Ok(())
    }
}

// Global lazy-initialized instance
#[allow(clippy::redundant_closure)]
static COSMOS_SERVICE: std::sync::LazyLock<
    Result<
        CosmosService,
        Box<dyn std::error::Error + std::marker::Send + std::marker::Sync + 'static>,
    >,
> = std::sync::LazyLock::new(|| CosmosService::new());

// Helper function to get the global instance
/// Returns a reference to the global Cosmos DB service instance.
///
/// # Errors
///
/// Returns an error if the Cosmos DB service failed to initialize.
#[allow(clippy::borrowed_box)]
pub fn get_cosmos_service() -> Result<
    &'static CosmosService,
    &'static Box<dyn std::error::Error + std::marker::Send + std::marker::Sync>,
> {
    COSMOS_SERVICE.as_ref()
}

/// Initialize the database and container on first access
///
/// # Errors
///
/// Returns an error if the Cosmos DB service cannot be initialized or accessed.
pub fn initialize_cosmos_db() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    get_cosmos_service().map_err(|e| format!("Failed to get Cosmos service: {e}"))?;

    Ok(())
}
