#[allow(unused_imports)]
use chrono::{DateTime, Duration, Utc};

use leptos::prelude::*;
use leptos::web_sys;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

//#[cfg(feature = "hydrate")]

#[derive(Clone)]
pub struct AuthContext {
    pub is_authenticated: ReadSignal<bool>,
    pub user_info: ReadSignal<Option<UserInfo>>,
    pub logout: Action<(), Result<(), String>>,
}

#[component]
#[allow(clippy::must_use_candidate)]
pub fn AuthProvider(children: Children) -> impl IntoView {
    let (is_authenticated, set_is_authenticated) = signal(false);
    let (user_info, set_user_info) = signal(None::<UserInfo>);

    // Check authentication status on mount
    let validate_auth = Resource::new(
        || (),
        |()| async move {
            #[cfg(feature = "hydrate")]
            {
                if let Some(token) = get_session_token() {
                    return validate_session(token).await;
                }
            }
            Ok::<AuthStatus, ServerFnError>(AuthStatus {
                is_authenticated: false,
                user_info: None,
                session_expires_in: None,
            })
        },
    );

    // Update authentication state when validation completes
    Effect::new(move |_| {
        if let Some(Ok(auth_status)) = validate_auth.get() {
            set_is_authenticated.set(auth_status.is_authenticated);
            set_user_info.set(auth_status.user_info);
        }
    });

    // Logout action
    let logout = Action::new(move |(): &()| async move {
        #[cfg(feature = "hydrate")]
        {
            if let Some(token) = get_session_token() {
                match logout_user(token).await {
                    Ok(_) => {
                        remove_session_token();
                        set_is_authenticated.set(false);
                        set_user_info.set(None);
                        Ok(())
                    }
                    Err(e) => Err(format!("Logout failed: {}", e)),
                }
            } else {
                set_is_authenticated.set(false);
                set_user_info.set(None);
                Ok(())
            }
        }
        #[cfg(not(feature = "hydrate"))]
        {
            set_is_authenticated.set(false);
            set_user_info.set(None);
            Ok(())
        }
    });

    let auth_context = AuthContext {
        is_authenticated,
        user_info,
        logout,
    };

    provide_context(auth_context);

    view! { {children()} }
}

#[must_use]
pub fn use_auth() -> AuthContext {
    expect_context::<AuthContext>()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub session_token: Option<String>,
    pub user_info: Option<UserInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub display_name: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub user_id: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    pub is_authenticated: bool,
    pub user_info: Option<UserInfo>,
    pub session_expires_in: Option<i64>, // seconds until expiration
}

// Simple in-memory session store (use Redis/Azure Cache in production)
#[allow(dead_code)]
static SESSION_STORE: std::sync::LazyLock<Mutex<HashMap<String, SessionInfo>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

#[server(AuthenticateUser, "/api")]
pub async fn authenticate_user(credentials: LoginRequest) -> Result<LoginResponse, ServerFnError> {
    // Extract the app config from Axum state
    use crate::config::AppConfig;
    use axum::extract::State;
    use leptos_axum::extract;
    use uuid::Uuid;
    let State(app_config): State<AppConfig> = extract()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to extract app config: {}", e)))?;

    // Validate credentials against configuration
    let is_valid = credentials.username == app_config.auth.username
        && credentials.password == app_config.auth.password;

    if is_valid {
        // Generate secure session token
        let session_token = format!("session_{}", Uuid::new_v4());
        let user_id = Uuid::new_v4().to_string();

        // Calculate session expiration
        let session_timeout_hours = app_config.auth.session_timeout_hours;
        let expires_at = Utc::now() + Duration::hours(session_timeout_hours as i64);

        // Create session info
        let session_info = SessionInfo {
            user_id: user_id.clone(),
            username: credentials.username.clone(),
            created_at: Utc::now(),
            expires_at,
            is_active: true,
        };

        // Store session in memory (use Azure Cache/Redis in production)
        {
            let mut sessions = SESSION_STORE.lock().unwrap();
            sessions.insert(session_token.clone(), session_info);
        }

        // Create user info
        let user_info = UserInfo {
            username: credentials.username.clone(),
            display_name: credentials.username.clone(), // In real app, get from user profile
            permissions: vec!["read_todos".to_string(), "write_todos".to_string()],
        };

        leptos::logging::log!(
            "User {} authenticated successfully with session {}",
            credentials.username,
            session_token
        );

        Ok(LoginResponse {
            success: true,
            message: "Authentication successful".to_string(),
            session_token: Some(session_token),
            user_info: Some(user_info),
        })
    } else {
        leptos::logging::log!("Authentication failed for user: {}", credentials.username);

        // Add delay to prevent brute force attacks
        #[cfg(feature = "ssr")]
        {
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }

        Ok(LoginResponse {
            success: false,
            message: "Invalid username or password".to_string(),
            session_token: None,
            user_info: None,
        })
    }
}

#[server(ValidateSession, "/api")]
pub async fn validate_session(session_token: String) -> Result<AuthStatus, ServerFnError> {
    let sessions = SESSION_STORE.lock().unwrap();

    if let Some(session_info) = sessions.get(&session_token) {
        // Check if session is still valid
        if session_info.is_active && Utc::now() < session_info.expires_at {
            let expires_in = (session_info.expires_at - Utc::now()).num_seconds();

            let user_info = UserInfo {
                username: session_info.username.clone(),
                display_name: session_info.username.clone(),
                permissions: vec!["read_todos".to_string(), "write_todos".to_string()],
            };

            Ok(AuthStatus {
                is_authenticated: true,
                user_info: Some(user_info),
                session_expires_in: Some(expires_in),
            })
        } else {
            // Session expired or inactive
            Ok(AuthStatus {
                is_authenticated: false,
                user_info: None,
                session_expires_in: None,
            })
        }
    } else {
        // Session not found
        Ok(AuthStatus {
            is_authenticated: false,
            user_info: None,
            session_expires_in: None,
        })
    }
}

#[server(LogoutUser, "/api")]
pub async fn logout_user(session_token: String) -> Result<bool, ServerFnError> {
    let mut sessions = SESSION_STORE.lock().unwrap();

    if let Some(session_info) = sessions.get_mut(&session_token) {
        session_info.is_active = false;
        leptos::logging::log!("User {} logged out", session_info.username);
        Ok(true)
    } else {
        Ok(false)
    }
}

#[server(RefreshSession, "/api")]
pub async fn refresh_session(session_token: String) -> Result<String, ServerFnError> {
    use crate::config::AppConfig;
    use axum::extract::State;
    use leptos_axum::extract;
    let State(app_config): State<AppConfig> = extract()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to extract app config: {}", e)))?;

    let mut sessions = SESSION_STORE.lock().unwrap();

    if let Some(session_info) = sessions.get_mut(&session_token) {
        if session_info.is_active && Utc::now() < session_info.expires_at {
            // Extend session
            let session_timeout_hours = app_config.auth.session_timeout_hours;
            session_info.expires_at = Utc::now() + Duration::hours(session_timeout_hours as i64);

            leptos::logging::log!("Session refreshed for user {}", session_info.username);
            Ok(session_token)
        } else {
            Err(ServerFnError::new("Session expired or invalid".to_string()))
        }
    } else {
        Err(ServerFnError::new("Session not found".to_string()))
    }
}

// Helper function to get session from browser storage
#[must_use]
pub fn get_session_token() -> Option<String> {
    use web_sys::window;

    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(token)) = storage.get_item("session_token") {
                return Some(token);
            }
        }
    }
    None
}

// Helper function to store session in browser storage
//#[cfg(feature = "hydrate")]
pub fn store_session_token(token: &str) {
    use web_sys::window;

    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("session_token", token);
        }
    }
}

// Helper function to remove session from browser storage
//#[cfg(feature = "hydrate")]
pub fn remove_session_token() {
    use web_sys::window;

    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("session_token");
        }
    }
}
