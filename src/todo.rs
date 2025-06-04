use chrono::{DateTime, Local, TimeZone};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use validator::Validate;
// Enhanced Todo struct with additional fields
#[derive(Clone, Debug, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TodoStatus {
    Pending,
    Completed,
}

impl TodoStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            TodoStatus::Pending => "Pending",
            TodoStatus::Completed => "Completed",
        }
    }
    #[must_use]
    pub fn bg_color(self) -> &'static str {
        match self {
            TodoStatus::Pending => "bg-gray-100 text-gray-800",
            TodoStatus::Completed => "bg-green-100 text-green-800",
        }
    }
}

impl FromStr for TodoStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Completed" => Ok(TodoStatus::Completed),
            _ => Ok(TodoStatus::Pending),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TodoAssignee {
    Mikko,
    Niina,
}
impl TodoAssignee {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            TodoAssignee::Mikko => "Mikko",
            TodoAssignee::Niina => "Niina",
        }
    }
}

impl Display for TodoAssignee {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for TodoAssignee {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "niina" => Ok(TodoAssignee::Niina),
            _ => Ok(TodoAssignee::Mikko), // Default to Mikko for unknown values
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Validate)]
pub struct Todo {
    pub id: String,
    #[validate(length(min = 1, max = 500, message = "Title must be 1-500 characters"))]
    #[validate(custom(function = "validate_no_html"))]
    pub title: String,
    #[validate(length(max = 2000, message = "Description too long"))]
    #[validate(custom(function = "validate_no_html"))]
    pub description: Option<String>,
    pub due_date: Option<u64>, // Unix timestamp in seconds
    pub assignee: TodoAssignee,
    pub status: TodoStatus,
}

fn validate_no_html(input: &str) -> Result<(), validator::ValidationError> {
    if input.contains('<') || input.contains('>') || input.contains('&') {
        return Err(validator::ValidationError::new("HTML tags not allowed"));
    }
    Ok(())
}

impl Todo {
    #[must_use]
    pub fn format_due_date(&self) -> String {
        match self.due_date {
            Some(timestamp) => {
                if let Ok(timestamp_i64) = i64::try_from(timestamp) {
                    if let Some(date) = DateTime::from_timestamp(timestamp_i64, 0) {
                        let local_date = Local.from_local_datetime(&date.naive_utc()).single();
                        match local_date {
                            Some(ld) => ld.format("%Y-%m-%d %H:%M").to_string(),
                            None => "Invalid date".to_string(),
                        }
                    } else {
                        "Invalid date".to_string()
                    }
                } else {
                    "Invalid date".to_string()
                }
            }
            None => "No deadline".to_string(),
        }
    }
    #[must_use]
    pub fn is_overdue(&self) -> bool {
        if matches!(self.status, TodoStatus::Completed) {
            return false;
        }

        match self.due_date {
            Some(deadline) => {
                let now = chrono::Local::now().timestamp();
                if now < 0 {
                    return false; // If current time is before epoch, not overdue
                }
                deadline < now.unsigned_abs()
            }
            None => false,
        }
    }
}
