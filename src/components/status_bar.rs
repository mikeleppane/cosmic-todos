use chrono::Local;
use leptos::prelude::*;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServerStatus {
    Online,
    Offline,
    Checking,
}

#[component]
#[allow(clippy::too_many_lines)]
#[must_use]
#[allow(clippy::must_use_candidate)]
pub fn StatusBar() -> impl IntoView {
    let (server_status, set_server_status) = signal(ServerStatus::Checking);
    let (last_successful_check, set_last_successful_check) = signal(Local::now());
    let (last_attempt, set_last_attempt) = signal(Local::now());
    let (is_mounted, set_is_mounted) = signal(true);

    // Create a heartbeat action
    let heartbeat_action = Action::new(move |(): &()| async move {
        use crate::api::heartbeat::heartbeat_server;
        heartbeat_server().await
    });

    // Safe signal update function that checks if component is still mounted
    let safe_update_status = move |status: ServerStatus| {
        if is_mounted.get_untracked() {
            set_server_status.set(status);
            set_last_attempt.set(Local::now());
            if status == ServerStatus::Online {
                set_last_successful_check.set(Local::now());
            }
        }
    };

    // Manual heartbeat check function
    let check_heartbeat = move || {
        if is_mounted.get_untracked() {
            set_server_status.set(ServerStatus::Checking);
            heartbeat_action.dispatch(());
        }
    };

    // Watch heartbeat action results with proper cleanup checking
    Effect::new(move |_| {
        if let Some(result) = heartbeat_action.value().get() {
            // Only update if component is still mounted
            if is_mounted.get_untracked() {
                match result {
                    Ok(_) => safe_update_status(ServerStatus::Online),
                    Err(_) => safe_update_status(ServerStatus::Offline),
                }
            }
        }
    });

    // Periodic heartbeat check with proper cleanup
    Effect::new(move |_| {
        let interval_id = if let Ok(id) = set_interval_with_handle(
            move || {
                if is_mounted.get_untracked() {
                    check_heartbeat();
                }
            },
            Duration::from_secs(30),
        ) {
            id
        } else {
            leptos::logging::warn!("Failed to set up interval for heartbeat check");
            return; // Exit the effect if we can't set up the interval
        };

        // Cleanup function with the extracted interval ID
        on_cleanup(move || {
            set_is_mounted.set(false);
            clear_interval(interval_id);
        });
    });

    // Initial heartbeat check
    Effect::new(move |_| {
        check_heartbeat();
    });

    // Status display helpers
    let status_color = move || {
        if !is_mounted.get_untracked() {
            return "bg-gray-500".to_string();
        }

        match server_status.get() {
            ServerStatus::Online => "bg-green-500".to_string(),
            ServerStatus::Offline => "bg-red-500".to_string(),
            ServerStatus::Checking => "bg-yellow-500".to_string(),
        }
    };

    let status_text = move || {
        if !is_mounted.get_untracked() {
            return "Disconnected".to_string();
        }

        match server_status.get() {
            ServerStatus::Online => "Server Online".to_string(),
            ServerStatus::Offline => "Server Offline".to_string(),
            ServerStatus::Checking => "Checking...".to_string(),
        }
    };

    let format_last_check = move || {
        if !is_mounted.get_untracked() {
            return "Unknown".to_string();
        }

        last_successful_check.get().format("%H:%M:%S").to_string()
    };

    view! {
        <div class="fixed bottom-4 right-4 z-40">
            <div class="bg-white rounded-lg shadow-lg border border-gray-200 p-3 min-w-[200px]">
                <div class="flex items-center gap-3">
                    // Status indicator dot with pulse animation for checking state
                    <div class="relative">
                        <div class=move || format!("w-3 h-3 rounded-full {}", status_color())></div>
                        <Show when=move || {
                            is_mounted.get_untracked() && server_status.get() == ServerStatus::Checking
                        }>
                            <div class="absolute inset-0 w-3 h-3 rounded-full bg-yellow-500 animate-ping opacity-75"></div>
                        </Show>
                    </div>

                    <div class="flex-1">
                        <p class="text-sm font-medium text-gray-900">{status_text}</p>
                        <Show when=move || {
                            is_mounted.get_untracked() && server_status.get() != ServerStatus::Checking
                        }>
                            <p class="text-xs text-gray-500">"Last check: " {format_last_check}</p>
                        </Show>
                    </div>

                    // Manual refresh button
                    <button
                        on:click=move |_| {
                            if is_mounted.get_untracked() {
                                leptos::logging::log!("Manual heartbeat check triggered");
                                check_heartbeat();
                            }
                        }
                        class="p-1 text-gray-400 hover:text-gray-600 rounded transition-colors"
                        title="Check server status"
                        disabled=move || {
                            !is_mounted.get_untracked() || server_status.get() == ServerStatus::Checking
                        }
                    >
                        <svg
                            class=move || {
                                format!(
                                    "w-4 h-4 {}",
                                    if is_mounted.get_untracked()
                                        && server_status.get() == ServerStatus::Checking
                                    {
                                        "animate-spin"
                                    } else {
                                        ""
                                    },
                                )
                            }
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                            />
                        </svg>
                    </button>
                </div>

                // Additional info when offline
                <Show when=move || is_mounted.get_untracked() && server_status.get() == ServerStatus::Offline>
                    <div class="mt-2 pt-2 border-t border-gray-100">
                        <p class="text-xs text-red-600">
                            "Server connection lost. Some features may not work."
                        </p>
                    </div>
                </Show>

                // Debug info (remove in production) - Now displayed vertically
                <Show when=move || is_mounted.get_untracked() && cfg!(debug_assertions)>
                    <div class="mt-2 pt-2 border-t border-gray-100">
                        <div class="space-y-1">
                            <p class="text-xs text-gray-500">
                                "Status: "
                                {move || {
                                    if is_mounted.get_untracked() {
                                        format!("{:?}", server_status.get())
                                    } else {
                                        "Unmounted".to_string()
                                    }
                                }}
                            </p>
                            <p class="text-xs text-gray-500">
                                "Last successful: "
                                {move || {
                                    if is_mounted.get_untracked() {
                                        last_successful_check.get().format("%H:%M:%S").to_string()
                                    } else {
                                        "N/A".to_string()
                                    }
                                }}
                            </p>
                            <p class="text-xs text-gray-500">
                                "Last attempt: "
                                {move || {
                                    if is_mounted.get_untracked() {
                                        last_attempt.get().format("%H:%M:%S").to_string()
                                    } else {
                                        "N/A".to_string()
                                    }
                                }}
                            </p>
                        </div>
                    </div>
                </Show>
            </div>
        </div>
    }
}

// Simplified interval handling that returns interval ID
#[cfg(feature = "hydrate")]
fn set_interval_with_handle<F>(f: F, duration: Duration) -> Result<i32, wasm_bindgen::JsValue>
where
    F: Fn() + 'static,
{
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;

    let closure = Closure::wrap(Box::new(f) as Box<dyn Fn()>);
    let function = closure.as_ref().unchecked_ref();

    let interval_id = web_sys::window()
        .unwrap()
        .set_interval_with_callback_and_timeout_and_arguments_0(
            function,
            duration.as_millis() as i32,
        )?;

    // Keep the closure alive by forgetting it
    closure.forget();

    Ok(interval_id)
}

#[cfg(not(feature = "hydrate"))]
#[allow(clippy::unnecessary_wraps)]
fn set_interval_with_handle<F>(_f: F, _duration: Duration) -> Result<i32, ()>
where
    F: Fn() + 'static,
{
    Ok(0) // Return dummy ID on server
}

#[cfg(feature = "hydrate")]
fn clear_interval(interval_id: i32) {
    if let Some(window) = web_sys::window() {
        window.clear_interval_with_handle(interval_id);
    }
}

#[cfg(not(feature = "hydrate"))]
fn clear_interval(_interval_id: i32) {
    // No-op on server
}
