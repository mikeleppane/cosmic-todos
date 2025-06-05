use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1, message = "Username is required"))]
    pub username: String,

    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub token: Option<String>,
    pub user_info: Option<UserInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub email: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthState {
    pub is_authenticated: bool,
    pub user_info: Option<UserInfo>,
    pub session_token: Option<String>,
}

impl AuthState {
    pub fn new() -> Self {
        Self {
            is_authenticated: false,
            user_info: None,
            session_token: None,
        }
    }

    pub fn authenticated(user_info: UserInfo, token: String) -> Self {
        Self {
            is_authenticated: true,
            user_info: Some(user_info),
            session_token: Some(token),
        }
    }

    pub fn logout(&mut self) {
        self.is_authenticated = false;
        self.user_info = None;
        self.session_token = None;
    }
}

impl Default for AuthState {
    fn default() -> Self {
        Self::new()
    }
}
