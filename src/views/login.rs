use leptos::{ev, prelude::*};
use leptos_router::{NavigateOptions, hooks::use_navigate};
use std::env;

#[component]
#[allow(clippy::too_many_lines)]
#[allow(clippy::must_use_candidate)]
pub fn LoginPage(set_authenticated: WriteSignal<bool>) -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(String::new());

    let handle_login = move |ev: ev::SubmitEvent| {
        ev.prevent_default();

        // In a production app, you would use a more secure way to handle credentials
        // These would be set through server environment variables
        let valid_username = env::var("COSMIC_USERNAME").unwrap_or_else(|_| "admin".to_string());
        let valid_password = env::var("COSMIC_PASSWORD").unwrap_or_else(|_| "password".to_string());

        if username.get() == valid_username && password.get() == valid_password {
            set_authenticated.set(true);
            let navigate = use_navigate();
            navigate("/todo", NavigateOptions::default());
        } else {
            set_error.set("Invalid username or password".to_string());
        }
    };

    view! {
        <main class="flex items-center justify-center min-h-screen bg-gradient-to-br from-fuchsia-100 via-sky-100 to-indigo-200">
            <div class="w-full max-w-md transform transition-all hover:scale-[1.02]">
                <div class="relative bg-white/90 backdrop-blur-sm rounded-2xl shadow-xl overflow-hidden border border-indigo-100">
                    // Decorative top border - more colorful gradient
                    <div class="absolute top-0 left-0 right-0 h-3 bg-gradient-to-r from-purple-500 via-fuchsia-500 to-indigo-500"></div>

                    // Add floating decoration elements
                    <div class="absolute -top-10 -right-10 w-32 h-32 rounded-full bg-gradient-to-br from-fuchsia-400/30 to-indigo-400/30 blur-xl"></div>
                    <div class="absolute -bottom-6 -left-6 w-24 h-24 rounded-full bg-gradient-to-tr from-sky-400/30 to-emerald-400/30 blur-xl"></div>

                    <div class="p-8 space-y-6">
                        <div class="text-center">
                            <div class="flex justify-center mb-4">
                                <div class="p-3 bg-gradient-to-r from-sky-100 to-indigo-100 rounded-full shadow-inner">
                                    <svg
                                        xmlns="http://www.w3.org/2000/svg"
                                        class="h-12 w-12 text-transparent bg-clip-text bg-gradient-to-r from-purple-600 to-indigo-600"
                                        fill="none"
                                        viewBox="0 0 24 24"
                                        stroke="currentColor"
                                    >
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            stroke-width="2"
                                            d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                                        />
                                    </svg>
                                </div>
                            </div>
                            <h1 class="text-3xl font-extrabold bg-clip-text text-transparent bg-gradient-to-r from-purple-600 via-fuchsia-600 to-indigo-600">
                                "Family Leppänen Todos"
                            </h1>
                            <p class="mt-2 text-gray-600 font-medium">
                                "Sign in to manage your tasks"
                            </p>
                        </div>

                        <form class="mt-6 space-y-5" on:submit=handle_login>
                            // More concise input fields with a colorful focus state
                            <div>
                                <input
                                    id="username"
                                    type="text"
                                    required
                                    class="block w-full px-4 py-3 bg-indigo-50/50 border-0 rounded-xl shadow-sm placeholder-indigo-400 focus:outline-none focus:ring-2 focus:ring-fuchsia-500 transition-all"
                                    prop:value=move || username.get()
                                    on:input=move |ev| set_username.set(event_target_value(&ev))
                                    placeholder="Username"
                                />
                            </div>

                            <div>
                                <input
                                    id="password"
                                    type="password"
                                    required
                                    class="block w-full px-4 py-3 bg-indigo-50/50 border-0 rounded-xl shadow-sm placeholder-indigo-400 focus:outline-none focus:ring-2 focus:ring-fuchsia-500 transition-all"
                                    prop:value=move || password.get()
                                    on:input=move |ev| set_password.set(event_target_value(&ev))
                                    placeholder="Password"
                                />
                            </div>

                            <Show when=move || !error.get().is_empty() fallback=|| "">
                                <div class="p-3 rounded-xl bg-red-50 border border-red-100 shadow-sm">
                                    <div class="flex items-center">
                                        <div class="flex-shrink-0">
                                            <svg
                                                xmlns="http://www.w3.org/2000/svg"
                                                class="h-5 w-5 text-red-400"
                                                viewBox="0 0 20 20"
                                                fill="currentColor"
                                            >
                                                <path
                                                    fill-rule="evenodd"
                                                    d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                                                    clip-rule="evenodd"
                                                />
                                            </svg>
                                        </div>
                                        <div class="ml-3">
                                            <p class="text-sm font-medium text-red-600">
                                                {move || error.get()}
                                            </p>
                                        </div>
                                    </div>
                                </div>
                            </Show>

                            <button
                                type="submit"
                                class="w-full flex justify-center py-3 px-4 border-0 rounded-xl shadow-md text-sm font-medium text-white bg-gradient-to-r from-purple-600 via-fuchsia-600 to-indigo-600 hover:from-purple-700 hover:via-fuchsia-700 hover:to-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-fuchsia-500 transition-all duration-300 transform hover:-translate-y-1 hover:shadow-lg"
                            >
                                "Sign In"
                            </button>
                        </form>

                        <div class="mt-5 text-center">
                            <div class="relative">
                                <div class="absolute inset-0 flex items-center">
                                    <div class="w-full border-t border-indigo-100"></div>
                                </div>
                                <div class="relative flex justify-center text-sm">
                                    <span class="px-2 bg-white text-indigo-500 font-medium">
                                        "Demo Info"
                                    </span>
                                </div>
                            </div>

                            <div class="mt-4 text-center">
                                <p class="text-sm text-gray-600 font-medium">
                                    "Default credentials:"
                                </p>
                                <div class="mt-2 flex justify-center space-x-3">
                                    <div class="inline-flex items-center px-3 py-1 rounded-full text-xs font-medium bg-gradient-to-r from-purple-100 to-indigo-100 text-indigo-800 shadow-sm">
                                        "admin"
                                    </div>
                                    <div class="inline-flex items-center px-3 py-1 rounded-full text-xs font-medium bg-gradient-to-r from-fuchsia-100 to-purple-100 text-fuchsia-800 shadow-sm">
                                        "password"
                                    </div>
                                </div>
                                <p class="mt-3 text-xs text-gray-500">
                                    "(Can be overridden with COSMIC_USERNAME and COSMIC_PASSWORD env variables)"
                                </p>
                            </div>
                        </div>
                    </div>
                </div>

                // Improved footer with gradient text
                <div class="mt-4 text-center">
                    <p class="text-xs font-medium bg-clip-text text-transparent bg-gradient-to-r from-purple-600 to-indigo-600">
                        "© 2025 Family Leppänen · All rights reserved"
                    </p>
                </div>
            </div>
        </main>
    }
}
