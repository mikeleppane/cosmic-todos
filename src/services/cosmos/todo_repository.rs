use azure_core::error::Error as AzureError;
use azure_data_cosmos::PartitionKey;
use futures::TryStreamExt;
use leptos::leptos_dom::logging;

use crate::{
    domain::todo::Todo,
    services::cosmos::{CosmosDBClient, model::CosmosDbTodo},
};

pub struct CosmosService {
    client: CosmosDBClient,
}

impl CosmosService {
    /// Creates a new instance of the Cosmos DB service.
    ///
    /// # Errors
    ///
    /// Returns an error if the app configuration cannot be retrieved or if the Cosmos client
    /// cannot be initialized with the provided connection details.
    pub fn new(client: CosmosDBClient) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self { client })
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
        let partition_key = PartitionKey::from("family_todos");
        match self
            .client
            .container()
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
        // Use a more explicit query approach
        let query =
            "SELECT * FROM c WHERE c.partition_key = 'family_todos' ORDER BY c.created_at DESC";
        let partition_key = PartitionKey::from("family_todos");

        logging::console_log("Starting Cosmos DB query for todos...");

        let mut todos = Vec::new();

        // Create the query stream
        let query_result =
            self.client
                .container()
                .query_items::<CosmosDbTodo>(query, partition_key, None);

        match query_result {
            Ok(mut query_stream) => {
                logging::console_log("Query stream created successfully");

                // Process the stream more carefully
                loop {
                    match query_stream.try_next().await {
                        Ok(Some(feed_page)) => {
                            logging::console_log(&format!(
                                "Received feed page with {} items",
                                feed_page.items().len()
                            ));

                            for item in feed_page.items() {
                                logging::console_log(&format!("Processing item: {item:#?}"));
                                todos.push(item.clone());
                            }
                        }
                        Ok(None) => {
                            break; // No more pages
                        }
                        Err(e) => {
                            logging::console_error(&format!(
                                "Error reading from query stream: {e}"
                            ));
                            return Err(e);
                        }
                    }
                }
            }
            Err(e) => {
                logging::console_error(&format!("Error creating query stream: {e}"));
                return Err(e);
            }
        }

        logging::console_log(&format!("Retrieved {} todos from Cosmos DB", todos.len()));
        Ok(todos)
    }

    /// Updates a todo item in the Cosmos DB container
    ///
    /// # Errors
    ///
    /// Returns an `AzureError` if the update operation fails or if there's an issue
    /// connecting to the Cosmos DB service.
    ///
    pub async fn update_todo(
        &self,
        updated_todo: Todo,
    ) -> Result<CosmosDbTodo, Box<dyn std::error::Error + Send + Sync>> {
        let partition_key = PartitionKey::from("family_todos");

        // First, fetch the existing item using a query to preserve created_at and notification fields
        let query = format!(
            "SELECT * FROM c WHERE c.id = '{}' AND c.partition_key = 'family_todos'",
            updated_todo.id
        );

        let query_result = self.client.container().query_items::<CosmosDbTodo>(
            &query,
            partition_key.clone(),
            None,
        );

        let mut existing_todo: Option<CosmosDbTodo> = None;

        match query_result {
            Ok(mut query_stream) => {
                if let Ok(Some(feed_page)) = query_stream.try_next().await {
                    if let Some(item) = feed_page.items().first() {
                        existing_todo = Some(item.clone());
                    }
                }
            }
            Err(e) => {
                logging::console_error(&format!("Error querying existing todo: {e}"));
                return Err(Box::new(e));
            }
        }

        // Create the updated todo
        let mut cosmos_todo = CosmosDbTodo::try_from_todo(updated_todo)?;

        // If we found the existing todo, preserve the original timestamps and notification fields
        if let Some(existing) = existing_todo {
            cosmos_todo.created_at = existing.created_at; // Preserve original creation time
            cosmos_todo.reminder_24h_sent = existing.reminder_24h_sent;
            cosmos_todo.final_reminder_sent = existing.final_reminder_sent;
            cosmos_todo.last_notification_time = existing.last_notification_time;
        }

        // Always update the modification time
        cosmos_todo.updated_at = chrono::Utc::now()
            .timestamp()
            .max(0)
            .try_into()
            .unwrap_or(0);

        // Replace the item in Cosmos DB
        let response = self
            .client
            .container()
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
        let partition_key = PartitionKey::from("family_todos");

        self.client
            .container()
            .delete_item(partition_key, todo_id, None)
            .await?;

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
> = std::sync::LazyLock::new(|| {
    use crate::services::cosmos::client::CosmosDBClient;
    let app_config =
        crate::config::get_config().map_err(|e| format!("Failed to get app config: {e}"))?;
    let client = CosmosDBClient::new(app_config)
        .map_err(|e| format!("Failed to create Cosmos DB client: {e}"))?;
    Ok(CosmosService::new(client).map_err(|e| format!("Failed to create Cosmos service: {e}"))?)
});

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
