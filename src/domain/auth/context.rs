use leptos::prelude::*;

use crate::api::{AuthStatus, authenticate_user};

use super::model::{AuthState, LoginRequest, LoginResponse, UserInfo};

#[derive(Clone)]
pub struct AuthContext {
    pub is_authenticated: ReadSignal<bool>,
    pub user_info: ReadSignal<Option<UserInfo>>,
    pub logout: Action<(), Result<(), String>>,
    pub login: Action<LoginRequest, Result<LoginResponse, ServerFnError>>,
    pub is_loading: ReadSignal<bool>,
}

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
                use crate::api::validate_session;
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
                                    remove_auth_state();
                                    leptos::logging::log!("Session expired");
                                    return Some(validation_result);
                                }
                            }
                            Err(e) => {
                                leptos::logging::log!("Session validation error: {}", e);
                                // Network error or server error - keep local state for now
                                remove_auth_state();
                                return Some(AuthStatus {
                                    is_authenticated: false,
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
                        remove_auth_state();
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
                        if let Some(token) = &response.token {
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
            use crate::api::logout_user;
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

//<Provider value=auth_context>
//            {children()}
//        </Provider>

pub fn use_auth() -> AuthContext {
    expect_context::<AuthContext>()
}

// localStorage helpers for auth state
#[cfg(feature = "hydrate")]
pub fn get_auth_state() -> Option<AuthState> {
    use leptos::leptos_dom::logging;
    use web_sys::window;

    let window = window()?;
    let storage = window.local_storage().ok()??;

    match storage.get_item("auth_state") {
        Ok(Some(auth_state_str)) => match serde_json::from_str::<AuthState>(&auth_state_str) {
            Ok(auth_state) => Some(auth_state),
            Err(e) => {
                logging::console_warn(&format!("Failed to parse auth state: {}", e));
                None
            }
        },
        Ok(None) => None,
        Err(e) => {
            logging::console_warn(&format!("Failed to get auth state: {:?}", e));
            None
        }
    }
}

#[cfg(not(feature = "hydrate"))]
pub fn get_auth_state() -> Option<AuthState> {
    None
}

#[cfg(feature = "hydrate")]
pub fn store_auth_state(auth_state: &AuthState) {
    use leptos::leptos_dom::logging;
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            match serde_json::to_string(auth_state) {
                Ok(auth_state_str) => {
                    if let Err(e) = storage.set_item("auth_state", &auth_state_str) {
                        logging::console_warn(&format!("Failed to store auth state: {:?}", e));
                    }
                }
                Err(e) => {
                    logging::console_warn(&format!("Failed to serialize auth state: {}", e));
                }
            }
        }
    }
}

#[cfg(not(feature = "hydrate"))]
pub fn store_auth_state(_auth_state: &AuthState) {
    // No-op on server
}

#[cfg(feature = "hydrate")]
pub fn remove_auth_state() {
    use leptos::leptos_dom::logging;
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Err(e) = storage.remove_item("auth_state") {
                logging::console_warn(&format!("Failed to remove auth state: {:?}", e));
            }
        }
    }
}

#[cfg(not(feature = "hydrate"))]
pub fn remove_auth_state() {
    // No-op on server
}

#[cfg(feature = "hydrate")]
pub fn get_session_token() -> Option<String> {
    get_auth_state().and_then(|state| state.session_token)
}

#[cfg(not(feature = "hydrate"))]
pub fn get_session_token() -> Option<String> {
    None
}
