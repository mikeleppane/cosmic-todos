use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::enums::{TodoAssignee, TodoStatus};
use super::validation::validate_no_html;

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Validate)]
pub struct Todo {
    pub id: String,

    #[validate(length(
        min = 1,
        max = 200,
        message = "Title must be between 1 and 200 characters"
    ))]
    #[validate(custom(function = "validate_no_html", message = "Title cannot contain HTML"))]
    pub title: String,

    #[validate(length(max = 1000, message = "Description cannot exceed 1000 characters"))]
    #[validate(custom(
        function = "validate_no_html",
        message = "Description cannot contain HTML"
    ))]
    pub description: Option<String>,

    pub due_date: Option<u64>,
    pub assignee: TodoAssignee,
    pub status: TodoStatus,
}

impl Todo {
    pub fn new(title: String, assignee: TodoAssignee) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            description: None,
            due_date: None,
            assignee,
            status: TodoStatus::Pending,
        }
    }

    pub fn with_description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    pub fn with_due_date(mut self, due_date: Option<u64>) -> Self {
        self.due_date = due_date;
        self
    }

    pub fn is_overdue(&self) -> bool {
        if let Some(due_timestamp) = self.due_date {
            if let Ok(timestamp_i64) = i64::try_from(due_timestamp) {
                if let Some(due_datetime) = DateTime::from_timestamp(timestamp_i64, 0) {
                    let now = Utc::now();
                    return now > due_datetime && self.status == TodoStatus::Pending;
                }
            }
        }
        false
    }

    pub fn formatted_due_date(&self) -> Option<String> {
        self.due_date.and_then(|timestamp| {
            i64::try_from(timestamp).ok().and_then(|ts| {
                DateTime::from_timestamp(ts, 0).map(|dt| {
                    let local_dt = dt.with_timezone(&Local);
                    local_dt.format("%A, %B %d, %Y at %I:%M %p").to_string()
                })
            })
        })
    }

    pub fn email(&self) -> &'static str {
        self.assignee.email()
    }
}
impl std::fmt::Display for Todo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Todo(id: {}, title: {}, status: {:?}, due_date: {:?}, assignee: {})",
            self.id, self.title, self.status, self.due_date, self.assignee
        )
    }
}
