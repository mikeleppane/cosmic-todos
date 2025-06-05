use crate::app::heartbeat_server;
use leptos::logging;
use leptos::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ServerStatus {
    Online,
    Offline,
    Checking,
}

#[component]
#[allow(clippy::must_use_candidate)]
#[allow(clippy::too_many_lines)]
pub fn StatusBar() -> impl IntoView {
    let (server_status, set_server_status) = signal(ServerStatus::Checking);
    let (last_successful_check, set_last_successful_check) = signal(chrono::Local::now());
    let (last_attempt, set_last_attempt) = signal(chrono::Local::now());

    // Heartbeat action with better error handling
    let heartbeat_action = Action::new(move |(): &()| async move {
        logging::log!("Attempting heartbeat...");
        let result = heartbeat_server().await;
        logging::log!("Heartbeat result: {:?}", result);
        result
    });

    // Check server status
    let check_heartbeat = move || {
        logging::log!("Starting heartbeat check");
        set_server_status.set(ServerStatus::Checking);
        set_last_attempt.set(chrono::Local::now());
        heartbeat_action.dispatch(());
    };

    // Watch for heartbeat results with better error handling
    Effect::new(move |_| {
        if let Some(result) = heartbeat_action.value().get() {
            logging::log!("Processing heartbeat result: {:?}", result);

            let now = chrono::Local::now();

            match result {
                Ok(response) => {
                    logging::log!("Heartbeat successful: {}", response);
                    set_server_status.set(ServerStatus::Online);
                    set_last_successful_check.set(now);
                }
                Err(e) => {
                    logging::log!("Heartbeat failed: {:?}", e);
                    set_server_status.set(ServerStatus::Offline);
                    // Don't update last_successful_check on failure
                }
            }
        }
    });

    // Set up interval for heartbeat checks (every 30 seconds)
    Effect::new(move |_| {
        use leptos::prelude::request_animation_frame;
        use leptos::wasm_bindgen::JsCast;
        use leptos::web_sys;

        // Create a flag to track if component is still mounted
        let (is_mounted, set_is_mounted) = signal(true);

        // Do initial check after a small delay
        let initial_check = check_heartbeat;
        request_animation_frame(move || {
            if is_mounted.get_untracked() {
                initial_check();
            }
        });

        // Clone necessary values for the interval closure
        let heartbeat_action_clone = heartbeat_action;
        let set_server_status_clone = set_server_status;
        let set_last_attempt_clone = set_last_attempt;
        let is_mounted_clone = is_mounted;

        let closure = leptos::wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            // Check if component is still mounted before executing
            if is_mounted_clone.get_untracked() {
                logging::log!("Interval triggered heartbeat");
                set_server_status_clone.set(ServerStatus::Checking);
                set_last_attempt_clone.set(chrono::Local::now());
                heartbeat_action_clone.dispatch(());
            } else {
                logging::log!("Component unmounted, skipping heartbeat");
            }
        }) as Box<dyn Fn()>);

        let interval_id = web_sys::window().and_then(|window| {
            window
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    30000, // 30 seconds
                )
                .ok()
        });

        // Store closure to prevent it from being dropped
        closure.forget();

        // Cleanup function
        on_cleanup(move || {
            logging::log!("StatusBar component unmounting, cleaning up interval");
            set_is_mounted.set(false);
            if let (Some(window), Some(id)) = (web_sys::window(), interval_id) {
                window.clear_interval_with_handle(id);
            }
        });
    });

    let status_text = move || match server_status.get() {
        ServerStatus::Online => "Server Online",
        ServerStatus::Offline => "Server Offline",
        ServerStatus::Checking => "Checking...",
    };

    let status_color = move || match server_status.get() {
        ServerStatus::Online => "bg-green-500",
        ServerStatus::Offline => "bg-red-500",
        ServerStatus::Checking => "bg-yellow-500",
    };

    let format_last_check = move || {
        let now = chrono::Local::now();
        let check_time = match server_status.get() {
            ServerStatus::Checking => last_attempt.get(),
            _ => last_successful_check.get(),
        };
        let diff = now.signed_duration_since(check_time);

        if diff.num_seconds() < 60 {
            format!("{}s ago", diff.num_seconds().max(0))
        } else if diff.num_minutes() < 60 {
            format!("{}m ago", diff.num_minutes())
        } else {
            format!("{}h ago", diff.num_hours())
        }
    };

    // ...existing code...

    view! {
        <div class="fixed bottom-4 right-4 z-40">
            <div class="bg-white rounded-lg shadow-lg border border-gray-200 p-3 min-w-[200px]">
                <div class="flex items-center gap-3">
                    // Status indicator dot with pulse animation for checking state
                    <div class="relative">
                        <div class=move || format!("w-3 h-3 rounded-full {}", status_color())></div>
                        <Show when=move || server_status.get() == ServerStatus::Checking>
                            <div class="absolute inset-0 w-3 h-3 rounded-full bg-yellow-500 animate-ping opacity-75"></div>
                        </Show>
                    </div>

                    <div class="flex-1">
                        <p class="text-sm font-medium text-gray-900">{status_text}</p>
                        <Show when=move || server_status.get() != ServerStatus::Checking>
                            <p class="text-xs text-gray-500">"Last check: " {format_last_check}</p>
                        </Show>
                    </div>

                    // Manual refresh button
                    <button
                        on:click=move |_| {
                            logging::log!("Manual heartbeat check triggered");
                            check_heartbeat();
                        }
                        class="p-1 text-gray-400 hover:text-gray-600 rounded transition-colors"
                        title="Check server status"
                        disabled=move || server_status.get() == ServerStatus::Checking
                    >
                        <svg
                            class=move || {
                                format!(
                                    "w-4 h-4 {}",
                                    if server_status.get() == ServerStatus::Checking {
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
                <Show when=move || server_status.get() == ServerStatus::Offline>
                    <div class="mt-2 pt-2 border-t border-gray-100">
                        <p class="text-xs text-red-600">
                            "Server connection lost. Some features may not work."
                        </p>
                    </div>
                </Show>

                // Debug info (remove in production) - Now displayed vertically
                <Show when=move || cfg!(debug_assertions)>
                    <div class="mt-2 pt-2 border-t border-gray-100">
                        <div class="space-y-1">
                            <p class="text-xs text-gray-500">
                                "Status: " {move || format!("{:?}", server_status.get())}
                            </p>
                            <p class="text-xs text-gray-500">
                                "Last successful: "
                                {move || last_successful_check.get().format("%H:%M:%S").to_string()}
                            </p>
                            <p class="text-xs text-gray-500">
                                "Last attempt: "
                                {move || last_attempt.get().format("%H:%M:%S").to_string()}
                            </p>
                        </div>
                    </div>
                </Show>
            </div>
        </div>
    }
}
