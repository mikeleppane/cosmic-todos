#[allow(unused_imports)]
use chrono::{DateTime, Duration, Utc};

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use validator::{Validate, ValidationError};
//#[cfg(feature = "hydrate")]

#[derive(Clone)]
pub struct AuthContext {
    pub is_authenticated: ReadSignal<bool>,
    pub user_info: ReadSignal<Option<UserInfo>>,
    pub logout: Action<(), Result<(), String>>,
    pub login: Action<LoginRequest, Result<LoginResponse, ServerFnError>>,
    pub is_loading: ReadSignal<bool>,
}

// Serializable version of auth state for localStorage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthState {
    pub is_authenticated: bool,
    pub user_info: Option<UserInfo>,
    pub session_token: Option<String>,
}

// localStorage helpers for auth state
#[cfg(feature = "hydrate")]
pub fn get_auth_state() -> Option<AuthState> {
    use web_sys::window;

    let window = window()?;
    let storage = window.local_storage().ok()??;

    match storage.get_item("auth_state") {
        Ok(Some(auth_state_str)) => {
            match serde_json::from_str(&auth_state_str) {
                Ok(auth_state) => {
                    leptos::logging::log!("Successfully retrieved auth state from localStorage");
                    Some(auth_state)
                }
                Err(e) => {
                    leptos::logging::log!("Failed to parse auth state from localStorage: {}", e);
                    // Clear corrupted data
                    let _ = storage.remove_item("auth_state");
                    None
                }
            }
        }
        Ok(None) => {
            leptos::logging::log!("No auth state found in localStorage");
            None
        }
        Err(e) => {
            leptos::logging::log!("Error accessing localStorage: {:?}", e);
            None
        }
    }
}

#[cfg(feature = "hydrate")]
pub fn store_auth_state(auth_state: &AuthState) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            match serde_json::to_string(auth_state) {
                Ok(auth_state_str) => match storage.set_item("auth_state", &auth_state_str) {
                    Ok(_) => {
                        leptos::logging::log!(
                            "Successfully stored auth state in localStorage: authenticated={}",
                            auth_state.is_authenticated
                        );
                    }
                    Err(e) => {
                        leptos::logging::log!(
                            "Failed to store auth state in localStorage: {:?}",
                            e
                        );
                    }
                },
                Err(e) => {
                    leptos::logging::log!("Failed to serialize auth state: {}", e);
                }
            }
        }
    }
}

#[cfg(feature = "hydrate")]
pub fn remove_auth_state() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            match storage.remove_item("auth_state") {
                Ok(_) => {
                    leptos::logging::log!("Successfully removed auth state from localStorage");
                }
                Err(e) => {
                    leptos::logging::log!("Failed to remove auth state from localStorage: {:?}", e);
                }
            }
        }
    }
}

// Server-side stubs
#[cfg(not(feature = "hydrate"))]
#[must_use]
pub fn get_auth_state() -> Option<AuthState> {
    None
}

#[cfg(not(feature = "hydrate"))]
pub fn store_auth_state(_auth_state: &AuthState) {}

#[cfg(not(feature = "hydrate"))]
pub fn remove_auth_state() {}

// Keep the existing session token helpers for backward compatibility
#[cfg(feature = "hydrate")]
pub fn get_session_token() -> Option<String> {
    get_auth_state().and_then(|state| state.session_token)
}

#[cfg(feature = "hydrate")]
pub fn store_session_token(token: &str) {
    let mut auth_state = get_auth_state().unwrap_or(AuthState {
        is_authenticated: false,
        user_info: None,
        session_token: None,
    });
    auth_state.session_token = Some(token.to_string());
    store_auth_state(&auth_state);
}

#[cfg(feature = "hydrate")]
pub fn remove_session_token() {
    remove_auth_state();
}

#[cfg(not(feature = "hydrate"))]
#[must_use]
pub fn get_session_token() -> Option<String> {
    None
}

#[cfg(not(feature = "hydrate"))]
pub fn store_session_token(_token: &str) {}

#[cfg(not(feature = "hydrate"))]
pub fn remove_session_token() {}

#[component]
#[allow(clippy::must_use_candidate)]
#[allow(clippy::too_many_lines)]
pub fn AuthProvider(children: Children) -> impl IntoView {
    // Initialize state from localStorage FIRST, before any async operations
    let initial_auth_state = {
        #[cfg(feature = "hydrate")]
        {
            get_auth_state().unwrap_or(AuthState {
                is_authenticated: false,
                user_info: None,
                session_token: None,
            })
        }
        #[cfg(not(feature = "hydrate"))]
        {
            AuthState {
                is_authenticated: false,
                user_info: None,
                session_token: None,
            }
        }
    };

    leptos::logging::log!(
        "Initial auth state from localStorage: authenticated={}",
        initial_auth_state.is_authenticated
    );

    let (is_authenticated, set_is_authenticated) = signal(initial_auth_state.is_authenticated);
    let (user_info, set_user_info) = signal(initial_auth_state.user_info.clone());
    let (is_loading, set_is_loading) = signal(true);

    // Only validate session if we have a stored session, don't clear state immediately
    let validate_auth: Resource<Option<AuthStatus>> = Resource::new(
        || (),
        |()| async move {
            #[cfg(feature = "hydrate")]
            {
                if let Some(auth_state) = get_auth_state() {
                    leptos::logging::log!("Found stored auth state, validating...");

                    if auth_state.is_authenticated && auth_state.session_token.is_some() {
                        let token = auth_state.session_token.unwrap();
                        leptos::logging::log!("Validating session token...");

                        match validate_session(token.clone()).await {
                            Ok(validation_result) => {
                                leptos::logging::log!(
                                    "Session validation result: authenticated={}",
                                    validation_result.is_authenticated
                                );

                                if validation_result.is_authenticated {
                                    // Session is still valid, update the stored state with fresh data
                                    let updated_auth_state = AuthState {
                                        is_authenticated: true,
                                        user_info: validation_result.user_info.clone(),
                                        session_token: Some(token),
                                    };
                                    store_auth_state(&updated_auth_state);
                                    return Some(validation_result);
                                } else {
                                    // Session expired, but don't clear immediately - let the effect handle it
                                    leptos::logging::log!("Session expired");
                                    return Some(validation_result);
                                }
                            }
                            Err(e) => {
                                leptos::logging::log!("Session validation error: {}", e);
                                // Network error or server error - keep local state for now
                                return Some(AuthStatus {
                                    is_authenticated: auth_state.is_authenticated,
                                    user_info: auth_state.user_info,
                                    session_expires_in: None,
                                });
                            }
                        }
                    } else {
                        leptos::logging::log!("No session token found in stored state");
                    }
                } else {
                    leptos::logging::log!("No stored auth state found");
                }
            }

            // No stored state or not hydrated
            None
        },
    );

    // Update authentication state when validation completes
    Effect::new(move |_| {
        match validate_auth.get() {
            Some(Some(auth_status)) => {
                leptos::logging::log!(
                    "Updating auth state from validation: authenticated={}",
                    auth_status.is_authenticated
                );

                set_is_authenticated.set(auth_status.is_authenticated);
                set_user_info.set(auth_status.user_info.clone());
                set_is_loading.set(false);

                // Only update localStorage if session is valid
                #[cfg(feature = "hydrate")]
                {
                    if auth_status.is_authenticated {
                        let auth_state = AuthState {
                            is_authenticated: auth_status.is_authenticated,
                            user_info: auth_status.user_info,
                            session_token: get_session_token(),
                        };
                        store_auth_state(&auth_state);
                    } else {
                        // Only remove if validation explicitly failed
                        remove_auth_state();
                    }
                }
            }
            Some(None) => {
                // Validation completed but no session to validate
                leptos::logging::log!("No session to validate, keeping current state");
                set_is_loading.set(false);

                // If we had no stored state, ensure we're not authenticated
                #[cfg(feature = "hydrate")]
                {
                    if get_auth_state().is_none() {
                        set_is_authenticated.set(false);
                        set_user_info.set(None);
                    }
                }
            }
            None => {
                // Still validating or on server side
                leptos::logging::log!("Validation still in progress or server side");
                #[cfg(not(feature = "hydrate"))]
                {
                    set_is_loading.set(false);
                }
            }
        }
    });

    // Login action
    let login = Action::new(move |credentials: &LoginRequest| {
        let credentials = credentials.clone();
        async move {
            leptos::logging::log!("Login action started for user: {}", credentials.username);

            match authenticate_user(credentials).await {
                Ok(response) => {
                    leptos::logging::log!("Login response received: success={}", response.success);

                    if response.success {
                        #[allow(unused_variables)]
                        if let Some(token) = &response.session_token {
                            // Update signals immediately
                            set_is_authenticated.set(true);
                            set_user_info.set(response.user_info.clone());

                            // Store complete auth state in localStorage
                            #[cfg(feature = "hydrate")]
                            {
                                let auth_state = AuthState {
                                    is_authenticated: true,
                                    user_info: response.user_info.clone(),
                                    session_token: Some(token.clone()),
                                };
                                store_auth_state(&auth_state);
                                leptos::logging::log!(
                                    "Auth state stored in localStorage after successful login"
                                );
                            }
                        }
                    }
                    Ok(response)
                }
                Err(e) => {
                    leptos::logging::log!("Login failed: {}", e);
                    set_is_authenticated.set(false);
                    set_user_info.set(None);

                    #[cfg(feature = "hydrate")]
                    {
                        remove_auth_state();
                    }

                    Err(e)
                }
            }
        }
    });

    // Logout action
    let logout = Action::new(move |(): &()| async move {
        leptos::logging::log!("Logout action started");

        #[cfg(feature = "hydrate")]
        {
            if let Some(auth_state) = get_auth_state() {
                if let Some(token) = auth_state.session_token {
                    match logout_user(token).await {
                        Ok(_) => {
                            leptos::logging::log!("Server logout successful");
                        }
                        Err(e) => {
                            leptos::logging::log!("Server logout failed: {}", e);
                        }
                    }
                }
            }

            // Clear localStorage
            remove_auth_state();
        }

        set_is_authenticated.set(false);
        set_user_info.set(None);
        leptos::logging::log!("User logged out successfully");

        Ok(())
    });

    let auth_context = AuthContext {
        is_authenticated,
        user_info,
        logout,
        login,
        is_loading,
    };

    provide_context(auth_context);

    view! { {children()} }
}

#[must_use]
pub fn use_auth() -> AuthContext {
    expect_context::<AuthContext>()
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 3, max = 32))]
    pub username: String,
    #[validate(length(min = 8, max = 128, message = "Password must be 8-128 characters"))]
    #[validate(custom(function = "validate_password_strength"))]
    pub password: String,
}

fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    let has_upper = password.chars().any(char::is_uppercase);
    let has_lower = password.chars().any(char::is_lowercase);
    let has_digit = password.chars().any(char::is_numeric);
    let has_special = password
        .chars()
        .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));

    if !(has_upper && has_lower && has_digit && has_special) {
        return Err(ValidationError::new(
            "Password must contain uppercase, lowercase, digit, and special character",
        ));
    }

    Ok(())
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
            let mut sessions = SESSION_STORE
                .lock()
                .expect("Failed to acquire session store lock");
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
    let sessions = SESSION_STORE
        .lock()
        .expect("Failed to acquire session store lock");

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
    let mut sessions = SESSION_STORE
        .lock()
        .expect("Failed to acquire session store lock");

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

    let mut sessions = SESSION_STORE
        .lock()
        .expect("Failed to acquire session store lock");

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
