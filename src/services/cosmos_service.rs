use azure_core::{credentials::Secret, error::Error as AzureError};
use azure_data_cosmos::{CosmosClient, PartitionKey};
use leptos::leptos_dom::logging;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

use crate::todo::{Todo, TodoAssignee, TodoStatus};

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
}

impl From<Todo> for CosmosDbTodo {
    fn from(todo: Todo) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            title: todo.title,
            description: todo.description,
            due_date: todo.due_date,
            assignee: todo.assignee.as_str().to_string(),
            status: todo.status.as_str().to_string(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<CosmosDbTodo> for Todo {
    fn from(cosmos_todo: CosmosDbTodo) -> Self {
        Self {
            id: cosmos_todo.id.parse().unwrap_or(0), // Convert string ID back to usize for UI
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
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        /* let _ = env::var("AZURE_COSMOS_CONNECTION_STRING")
        .map_err(|_| "AZURE_COSMOS_CONNECTION_STRING environment variable not found")?; */

        let client = CosmosClient::with_key(
            "https://familyleppanen-cosmic-rust.documents.azure.com:443/",
            Secret::from(
                "FZWpWvjxjsAsmbRWMjFQP99R26ov994hii8uKa1ggGLf2pPezP3rWIIhhr4i3LhTBq38t8WBxb6ZACDbBe1meQ==",
            ),
            None,
        )?;

        Ok(Self {
            client,
            database_name: "familyleppanen".to_string(),
            container_name: "todos".to_string(),
        })
    }

    pub async fn create_todo(&self, todo: Todo) -> Result<Todo, AzureError> {
        let todo2 = todo.clone();
        let cosmos_todo = CosmosDbTodo::from(todo);

        let database = self.client.database_client(&self.database_name);
        let container = database.container_client(&self.container_name);
        let partition_key = PartitionKey::from(cosmos_todo.id.clone());
        match container
            .create_item(partition_key, cosmos_todo, None)
            .await
        {
            Ok(_) => {
                logging::console_log("SUCCESS");
                //let value: CosmosDbTodo = response.into_json_body().await?;
                logging::console_log("Created todo in Cosmos DB: {:?}");
                Ok(todo2)
            }
            Err(e) => {
                logging::console_error("ERROR");
                eprintln!("Error creating todo in Cosmos DB: {}", e);
                Err(e)
            }
        }
    }

    /* pub async fn get_todos(&self, family_id: &str) -> Result<Vec<CosmosDbTodo>, AzureError> {
        let database = self.client.database_client(&self.database_name);
        let container = database.container_client(&self.container_name);

        let query = "SELECT * FROM c WHERE c.family_id = @family_id ORDER BY c.created_at DESC";
        let query_params = vec![("@family_id", family_id)];

        let mut query_stream = container
            .query_items::<CosmosDbTodo>(query, &query_params)
            .max_item_count(100)
            .into_stream();

        let mut todos = Vec::new();

        while let Some(response) = query_stream.next().await {
            let response = response?;
            for item in response.results {
                todos.push(item);
            }
        }

        Ok(todos)
    } */

    /* pub async fn update_todo(
        &self,
        todo_id: &str,
        updated_todo: Todo,
    ) -> Result<CosmosDbTodo, AzureError> {
        let mut cosmos_todo = CosmosDbTodo::from(updated_todo);
        cosmos_todo.id = todo_id.to_string();
        cosmos_todo.updated_at = chrono::Utc::now().timestamp();

        let database = self.client.database_client(&self.database_name);
        let container = database.container_client(&self.container_name);

        let response = container
            .replace_item(&cosmos_todo.family_id, &cosmos_todo.id, &cosmos_todo)
            .await?;

        Ok(response.item)
    }

    pub async fn delete_todo(&self, family_id: &str, todo_id: &str) -> Result<(), AzureError> {
        let database = self.client.database_client(&self.database_name);
        let container = database.container_client(&self.container_name);

        container.delete_item(family_id, todo_id).await?;

        Ok(())
    } */
}

// Global lazy-initialized instance
static COSMOS_SERVICE: Lazy<Result<CosmosService, Box<dyn std::error::Error + Send + Sync>>> =
    Lazy::new(|| CosmosService::new());

// Helper function to get the global instance
pub fn get_cosmos_service()
-> Result<&'static CosmosService, &'static Box<dyn std::error::Error + Send + Sync>> {
    COSMOS_SERVICE.as_ref()
}

// Initialize the database and container on first access
pub fn initialize_cosmos_db() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    get_cosmos_service().map_err(|e| format!("Failed to get Cosmos service: {}", e))?;

    Ok(())
}
