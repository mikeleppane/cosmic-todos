use std::fmt::Display;
use std::str::FromStr;

use chrono::{DateTime, Local, TimeZone};
use serde::{Deserialize, Serialize};
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

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<i64>, // Unix timestamp in seconds
    pub assignee: TodoAssignee,
    pub status: TodoStatus,
}

impl Todo {
    #[must_use]
    pub fn format_due_date(&self) -> String {
        match self.due_date {
            Some(timestamp) => {
                if let Some(date) = DateTime::from_timestamp(timestamp, 0) {
                    let local_date = Local.from_local_datetime(&date.naive_utc()).single();
                    match local_date {
                        Some(ld) => ld.format("%Y-%m-%d %H:%M").to_string(),
                        None => "Invalid date".to_string(),
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
                deadline < now
            }
            None => false,
        }
    }
}
