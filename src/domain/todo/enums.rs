use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Clone, Debug, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TodoStatus {
    Pending,
    Completed,
}

impl TodoStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Completed => "Completed",
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

impl Display for TodoStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for TodoStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(Self::Pending),
            "Completed" => Ok(Self::Completed),
            _ => Err(format!("Invalid todo status: {}", s)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TodoAssignee {
    Mikko,
    Niina,
}

impl TodoAssignee {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mikko => "Mikko",
            Self::Niina => "Niina",
        }
    }

    pub fn email(&self) -> &'static str {
        match self {
            Self::Mikko => "mikko@familyleppanen.com",
            Self::Niina => "niina@familyleppanen.com",
        }
    }
}

impl Display for TodoAssignee {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for TodoAssignee {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Mikko" => Ok(Self::Mikko),
            "Niina" => Ok(Self::Niina),
            _ => Err(format!("Invalid assignee: {}", s)),
        }
    }
}
