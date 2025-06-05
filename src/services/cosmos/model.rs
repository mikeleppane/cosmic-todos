use std::str::FromStr;

use crate::{
    config::get_config,
    domain::todo::{Todo, TodoAssignee, TodoStatus},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosDbTodo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<u64>,
    pub assignee: String,
    pub status: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub partition_key: String,
    pub email: String,
    // Optional notification tracking fields for Azure Functions
    #[serde(skip_serializing_if = "Option::is_none", default = "default_false")]
    pub reminder_24h_sent: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none", default = "default_false")]
    pub final_reminder_sent: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none", default = "default_none")]
    pub last_notification_time: Option<i64>,
}

// Helper functions for default values
fn default_false() -> Option<bool> {
    None
}

fn default_none() -> Option<i64> {
    None
}

impl CosmosDbTodo {
    /// Converts a `Todo` into a `CosmosDbTodo` for database storage.
    ///
    /// # Errors
    ///
    /// Returns an error if the app configuration cannot be retrieved or if the
    /// assignee email is not found in the configuration.
    pub fn try_from_todo(todo: Todo) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let now = chrono::Utc::now()
            .timestamp()
            .max(0)
            .try_into()
            .unwrap_or(0);
        let config = get_config().map_err(|e| format!("Failed to get app config: {e}"))?;
        let email = config
            .emails
            .get(&todo.assignee)
            .ok_or("Assignee email not found")?;

        let due_date = todo.due_date; // No conversion needed, already u64

        Ok(Self {
            id: todo.id,
            title: todo.title,
            description: todo.description,
            due_date,
            assignee: todo.assignee.as_str().to_string(),
            status: todo.status.as_str().to_string(),
            created_at: now,
            updated_at: now,
            partition_key: "family_todos".to_string(),
            email: email.clone(),
            reminder_24h_sent: None,
            final_reminder_sent: None,
            last_notification_time: None,
        })
    }
}

impl From<CosmosDbTodo> for Todo {
    fn from(cosmos_todo: CosmosDbTodo) -> Self {
        Self {
            id: cosmos_todo.id.parse().unwrap_or(String::new()), // Convert string ID back to usize for UI
            title: cosmos_todo.title,
            description: cosmos_todo.description,
            due_date: Some(cosmos_todo.due_date.unwrap_or(0)), // Convert u64 back to i64 for UI
            assignee: TodoAssignee::from_str(&cosmos_todo.assignee).unwrap_or(TodoAssignee::Mikko),
            status: TodoStatus::from_str(&cosmos_todo.status).unwrap_or(TodoStatus::Pending),
        }
    }
}
