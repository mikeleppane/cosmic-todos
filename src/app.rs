#![allow(clippy::must_use_candidate)]
use leptos::{ev, prelude::*};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::use_navigate,
    NavigateOptions, StaticSegment,
};
use std::env;

// Static configuration loaded once at startup

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};

// Enhanced Todo struct with additional fields
#[derive(Clone, Debug, Copy)]
enum TodoStatus {
    NotStarted,
    InProgress,
    Completed,
    Blocked,
}

impl TodoStatus {
    fn as_str(self) -> &'static str {
        match self {
            TodoStatus::NotStarted => "Not Started",
            TodoStatus::InProgress => "In Progress",
            TodoStatus::Completed => "Completed",
            TodoStatus::Blocked => "Blocked",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "In Progress" => TodoStatus::InProgress,
            "Completed" => TodoStatus::Completed,
            "Blocked" => TodoStatus::Blocked,
            _ => TodoStatus::NotStarted,
        }
    }

    fn bg_color(self) -> &'static str {
        match self {
            TodoStatus::NotStarted => "bg-gray-100 text-gray-800",
            TodoStatus::InProgress => "bg-blue-100 text-blue-800",
            TodoStatus::Completed => "bg-green-100 text-green-800",
            TodoStatus::Blocked => "bg-red-100 text-red-800",
        }
    }
}

#[derive(Clone, Debug)]
enum TodoAssignee {
    Mikko,
    Niina,
}

impl TodoAssignee {
    fn as_str(&self) -> &'static str {
        match self {
            TodoAssignee::Mikko => "Mikko",
            TodoAssignee::Niina => "Niina",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "Niina" => TodoAssignee::Niina,
            _ => TodoAssignee::Mikko,
        }
    }
}

#[derive(Clone, Debug)]
struct Todo {
    id: usize,
    title: String,
    description: Option<String>,
    due_date: Option<i64>, // Unix timestamp in seconds
    assignee: TodoAssignee,
    status: TodoStatus,
}

impl Todo {
    fn format_due_date(&self) -> String {
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

    fn is_overdue(&self) -> bool {
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

#[must_use]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body class="bg-gray-100 min-h-screen">
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // Create an authentication state that can be shared across components
    let (authenticated, set_authenticated) = signal(false);

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/cosmic-rust.css" />

        // sets the document title
        <Title text="Family Leppänen Todos" />

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.">
                    <Route
                        path=StaticSegment("")
                        view=move || {
                            if authenticated.get() {
                                view! { <HomePage /> }.into_any()
                            } else {
                                view! { <LoginPage set_authenticated /> }.into_any()
                            }
                        }
                    />
                    <Route
                        path=StaticSegment("login")
                        view=move || view! { <LoginPage set_authenticated /> }
                    />
                    <Route
                        path=StaticSegment("todo")
                        view=move || {
                            if authenticated.get() {
                                view! { <HomePage /> }.into_any()
                            } else {
                                view! { <Redirect path="/login" /> }.into_any()
                            }
                        }
                    />
                </Routes>
            </main>
        </Router>
    }
}

/// Redirects to a different page
#[component]
fn Redirect(path: &'static str) -> impl IntoView {
    let navigate = use_navigate();

    Effect::new(move |_| {
        navigate(path, NavigateOptions::default());
    });
}

/// Login page component
#[component]
fn LoginPage(set_authenticated: WriteSignal<bool>) -> impl IntoView {
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

/// Renders the home page with the Todo application
#[component]
fn HomePage() -> impl IntoView {
    let (todos, set_todos) = signal(Vec::<Todo>::new());

    // Modal state
    let (show_modal, set_show_modal) = signal(false);

    // New todo form fields
    let (new_title, set_new_title) = signal(String::new());
    let (new_description, set_new_description) = signal(String::new());
    let (new_due_date, set_new_due_date) = signal(String::new());
    let (new_due_time, set_new_due_time) = signal(String::new());
    let (new_assignee, set_new_assignee) = signal("Mikko".to_string());
    let (new_status, set_new_status) = signal("Not Started".to_string());

    // Helper to reset form
    let reset_form = move || {
        set_new_title.set(String::new());
        set_new_description.set(String::new());
        set_new_due_date.set(String::new());
        set_new_due_time.set(String::new());
        set_new_assignee.set("Mikko".to_string());
        set_new_status.set("Not Started".to_string());
    };

    // Open modal handler
    let open_modal = move |_| {
        reset_form();
        set_show_modal.set(true);
    };

    // Close modal handler
    let close_modal = move |_| {
        set_show_modal.set(false);
    };

    // Add todo handler
    let add_todo = move |ev: ev::SubmitEvent| {
        ev.prevent_default();

        let title = new_title.get();
        if title.trim().is_empty() {
            return;
        }

        // Parse due date and time
        let due_timestamp = if new_due_date.get().is_empty() {
            None
        } else {
            let date_str = new_due_date.get();
            let time_str = if new_due_time.get().is_empty() {
                "00:00"
            } else {
                &new_due_time.get()
            };
            let datetime_str = format!("{date_str} {time_str}");

            match NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M") {
                Ok(dt) => Some(
                    Local
                        .from_local_datetime(&dt)
                        .single()
                        .map_or(0, |dt| dt.timestamp()),
                ),
                Err(_) => None,
            }
        };

        // Create new todo
        let new_todo = Todo {
            id: todos.get().len(),
            title: title.trim().to_string(),
            description: if new_description.get().trim().is_empty() {
                None
            } else {
                Some(new_description.get().trim().to_string())
            },
            due_date: due_timestamp,
            assignee: TodoAssignee::from_str(&new_assignee.get()),
            status: TodoStatus::from_str(&new_status.get()),
        };

        // Add to list
        set_todos.update(|todos| {
            todos.push(new_todo);
        });

        // Close modal
        set_show_modal.set(false);
    };

    // Toggle status handler
    let update_status = move |id: usize, new_status: &str| {
        set_todos.update(|todos| {
            if let Some(todo) = todos.iter_mut().find(|t| t.id == id) {
                todo.status = TodoStatus::from_str(new_status);
            }
        });
    };

    // Delete todo handler
    let delete_todo = move |id: usize| {
        set_todos.update(|todos| {
            todos.retain(|t| t.id != id);
        });
    };

    view! {
        <main class="my-0 mx-auto max-w-4xl p-6">
            <div class="relative bg-white rounded-2xl shadow-xl overflow-hidden border border-indigo-100">
                // Decorative top border - colorful gradient
                <div class="absolute top-0 left-0 right-0 h-3 bg-gradient-to-r from-purple-500 via-fuchsia-500 to-indigo-500"></div>

                <div class="px-6 py-5 bg-gradient-to-r from-purple-600 via-fuchsia-600 to-indigo-600 flex justify-between items-center">
                    <h1 class="text-2xl font-bold text-white">Family Leppänen Todos</h1>
                    <button
                        on:click=open_modal
                        class="px-4 py-2 bg-white/90 rounded-xl font-medium text-indigo-600 hover:bg-white/100 transition-all duration-200 hover:shadow-lg flex items-center space-x-2"
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            class="h-5 w-5"
                            viewBox="0 0 20 20"
                            fill="currentColor"
                        >
                            <path
                                fill-rule="evenodd"
                                d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z"
                                clip-rule="evenodd"
                            />
                        </svg>
                        <span>"New Task"</span>
                    </button>
                </div>

                // Main content
                <div class="p-6">
                    {move || {
                        if todos.get().is_empty() {
                            view! {
                                <div class="flex flex-col items-center justify-center py-16 text-center">
                                    <svg
                                        xmlns="http://www.w3.org/2000/svg"
                                        class="h-16 w-16 text-gray-300 mb-4"
                                        fill="none"
                                        viewBox="0 0 24 24"
                                        stroke="currentColor"
                                    >
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            stroke-width="1"
                                            d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
                                        />
                                    </svg>
                                    <h2 class="text-xl font-medium text-gray-600 mb-2">
                                        No tasks yet
                                    </h2>
                                    <p class="text-gray-500 mb-6">
                                        Create your first task to get started
                                    </p>
                                    <button
                                        on:click=open_modal
                                        class="px-4 py-2 bg-gradient-to-r from-purple-600 via-fuchsia-600 to-indigo-600 rounded-xl font-medium text-white hover:from-purple-700 hover:via-fuchsia-700 hover:to-indigo-700 transition-all duration-200 hover:shadow-lg flex items-center space-x-2"
                                    >
                                        <svg
                                            xmlns="http://www.w3.org/2000/svg"
                                            class="h-5 w-5"
                                            viewBox="0 0 20 20"
                                            fill="currentColor"
                                        >
                                            <path
                                                fill-rule="evenodd"
                                                d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z"
                                                clip-rule="evenodd"
                                            />
                                        </svg>
                                        <span>"Create New Task"</span>
                                    </button>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <div class="space-y-4">
                                    {todos
                                        .get()
                                        .into_iter()
                                        .map(|todo| {
                                            let id = todo.id;
                                            let is_overdue = todo.is_overdue();
                                            let status_options = vec![
                                                "Not Started",
                                                "In Progress",
                                                "Completed",
                                                "Blocked",
                                            ];
                                            let current_status = todo.status.as_str().to_string();
                                            let due_date = todo.format_due_date();

                                            view! {
                                                <div class="border border-gray-200 rounded-xl overflow-hidden hover:shadow-md transition-shadow">
                                                    <div class=format!(
                                                        "px-5 py-4 flex justify-between items-start {}",
                                                        if is_overdue
                                                            && !matches!(todo.status, TodoStatus::Completed)
                                                        {
                                                            "bg-red-50/50"
                                                        } else {
                                                            ""
                                                        },
                                                    )>
                                                        <div class="space-y-2">
                                                            <div class="flex items-center space-x-2">
                                                                <h3 class="font-medium text-lg">{todo.title.clone()}</h3>
                                                                <div class=format!(
                                                                    "px-2 py-1 text-xs font-medium rounded-full {}",
                                                                    todo.status.bg_color(),
                                                                )>{todo.status.as_str()}</div>
                                                                {move || {
                                                                    if is_overdue
                                                                        && !matches!(todo.status, TodoStatus::Completed)
                                                                    {
                                                                        view! {
                                                                            <span class="text-xs text-red-600 font-medium px-2 py-1 rounded-full bg-red-100">
                                                                                "Overdue"
                                                                            </span>
                                                                        }
                                                                            .into_any()
                                                                    } else {
                                                                        view! { <></> }
                                                                        ().into_any()
                                                                    }
                                                                }}
                                                            </div>

                                                            {move || {
                                                                let desc_option = todo.description.clone();
                                                                match desc_option {
                                                                    Some(desc) => {
                                                                        view! { <p class="text-gray-600 text-sm">{desc}</p> }
                                                                            .into_any()
                                                                    }
                                                                    None => ().into_any(),
                                                                }
                                                            }}

                                                            <div class="flex items-center space-x-4 text-sm text-gray-500">
                                                                <div class="flex items-center space-x-1">
                                                                    <svg
                                                                        xmlns="http://www.w3.org/2000/svg"
                                                                        class="h-4 w-4"
                                                                        fill="none"
                                                                        viewBox="0 0 24 24"
                                                                        stroke="currentColor"
                                                                    >
                                                                        <path
                                                                            stroke-linecap="round"
                                                                            stroke-linejoin="round"
                                                                            stroke-width="2"
                                                                            d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
                                                                        />
                                                                    </svg>
                                                                    <span class=move || {
                                                                        let mut classes = String::new();
                                                                        if is_overdue
                                                                            && !matches!(todo.status, TodoStatus::Completed)
                                                                        {
                                                                            classes.push_str("text-red-600 font-medium");
                                                                        }
                                                                        classes
                                                                    }>{due_date}</span>
                                                                </div>

                                                                <div class="flex items-center space-x-1">
                                                                    <svg
                                                                        xmlns="http://www.w3.org/2000/svg"
                                                                        class="h-4 w-4"
                                                                        fill="none"
                                                                        viewBox="0 0 24 24"
                                                                        stroke="currentColor"
                                                                    >
                                                                        <path
                                                                            stroke-linecap="round"
                                                                            stroke-linejoin="round"
                                                                            stroke-width="2"
                                                                            d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
                                                                        />
                                                                    </svg>
                                                                    <span>{todo.assignee.as_str()}</span>
                                                                </div>
                                                            </div>
                                                        </div>

                                                        <div class="flex space-x-2">
                                                            <select
                                                                class="text-sm rounded-lg border-gray-300 focus:border-indigo-500 focus:ring focus:ring-indigo-200 focus:ring-opacity-50"
                                                                on:change=move |ev| update_status(
                                                                    id,
                                                                    &event_target_value(&ev),
                                                                )
                                                                prop:value=current_status
                                                            >
                                                                {status_options
                                                                    .into_iter()
                                                                    .map(|status| {
                                                                        view! {
                                                                            <option value=status selected=status == current_status>
                                                                                {status}
                                                                            </option>
                                                                        }
                                                                    })
                                                                    .collect::<Vec<_>>()}
                                                            </select>

                                                            <button
                                                                on:click=move |_| delete_todo(id)
                                                                class="text-gray-400 hover:text-red-500 transition-colors"
                                                            >
                                                                <svg
                                                                    xmlns="http://www.w3.org/2000/svg"
                                                                    class="h-5 w-5"
                                                                    fill="none"
                                                                    viewBox="0 0 24 24"
                                                                    stroke="currentColor"
                                                                >
                                                                    <path
                                                                        stroke-linecap="round"
                                                                        stroke-linejoin="round"
                                                                        stroke-width="2"
                                                                        d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                                                                    />
                                                                </svg>
                                                            </button>
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                                </div>
                            }
                                .into_any()
                        }
                    }}
                </div>
            </div>

            // Modal
            <Show when=move || show_modal.get() fallback=|| ()>
                <div class="fixed inset-0 bg-gray-900/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
                    <div class="bg-white rounded-2xl shadow-2xl max-w-md w-full transform transition-all animate-fade-in-up">
                        <form on:submit=add_todo>
                            <div class="px-6 py-4 border-b border-gray-200">
                                <h2 class="text-xl font-semibold text-gray-800">Create New Task</h2>
                            </div>

                            <div class="px-6 py-4 space-y-4">
                                <div>
                                    <label
                                        for="title"
                                        class="block text-sm font-medium text-gray-700 mb-1"
                                    >
                                        Title*
                                    </label>
                                    <input
                                        id="title"
                                        type="text"
                                        required
                                        prop:value=move || new_title.get()
                                        on:input=move |ev| {
                                            set_new_title.set(event_target_value(&ev));
                                        }
                                        class="block w-full px-4 py-3 bg-gray-50 border-0 rounded-xl shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-fuchsia-500 transition-all"
                                        placeholder="Enter task title"
                                    />
                                </div>

                                <div>
                                    <label
                                        for="description"
                                        class="block text-sm font-medium text-gray-700 mb-1"
                                    >
                                        Description (optional)
                                    </label>
                                    <textarea
                                        id="description"
                                        prop:value=move || new_description.get()
                                        on:input=move |ev| {
                                            set_new_description.set(event_target_value(&ev));
                                        }
                                        class="block w-full px-4 py-3 bg-gray-50 border-0 rounded-xl shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-fuchsia-500 transition-all"
                                        rows="3"
                                        placeholder="Enter task description"
                                    ></textarea>
                                </div>

                                <div class="grid grid-cols-2 gap-4">
                                    <div>
                                        <label
                                            for="due-date"
                                            class="block text-sm font-medium text-gray-700 mb-1"
                                        >
                                            Due Date
                                        </label>
                                        <input
                                            id="due-date"
                                            type="date"
                                            prop:value=move || new_due_date.get()
                                            on:input=move |ev| {
                                                set_new_due_date.set(event_target_value(&ev));
                                            }
                                            class="block w-full px-4 py-3 bg-gray-50 border-0 rounded-xl shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-fuchsia-500 transition-all"
                                        />
                                    </div>
                                    <div>
                                        <label
                                            for="due-time"
                                            class="block text-sm font-medium text-gray-700 mb-1"
                                        >
                                            Due Time
                                        </label>
                                        <input
                                            id="due-time"
                                            type="time"
                                            prop:value=move || new_due_time.get()
                                            on:input=move |ev| {
                                                set_new_due_time.set(event_target_value(&ev));
                                            }
                                            class="block w-full px-4 py-3 bg-gray-50 border-0 rounded-xl shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-fuchsia-500 transition-all"
                                        />
                                    </div>
                                </div>

                                <div class="grid grid-cols-2 gap-4">
                                    <div>
                                        <label
                                            for="assignee"
                                            class="block text-sm font-medium text-gray-700 mb-1"
                                        >
                                            Assignee
                                        </label>
                                        <select
                                            id="assignee"
                                            prop:value=move || new_assignee.get()
                                            on:change=move |ev| {
                                                set_new_assignee.set(event_target_value(&ev));
                                            }
                                            class="block w-full px-4 py-3 bg-gray-50 border-0 rounded-xl shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-fuchsia-500 transition-all"
                                        >
                                            <option value="Mikko">Mikko</option>
                                            <option value="Niina">Niina</option>
                                        </select>
                                    </div>
                                    <div>
                                        <label
                                            for="status"
                                            class="block text-sm font-medium text-gray-700 mb-1"
                                        >
                                            Status
                                        </label>
                                        <select
                                            id="status"
                                            prop:value=move || new_status.get()
                                            on:change=move |ev| {
                                                set_new_status.set(event_target_value(&ev));
                                            }
                                            class="block w-full px-4 py-3 bg-gray-50 border-0 rounded-xl shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-fuchsia-500 transition-all"
                                        >
                                            <option value="Not Started">Not Started</option>
                                            <option value="In Progress">In Progress</option>
                                            <option value="Completed">Completed</option>
                                            <option value="Blocked">Blocked</option>
                                        </select>
                                    </div>
                                </div>
                            </div>

                            <div class="px-6 py-4 bg-gray-50 flex justify-end space-x-3 rounded-b-2xl">
                                <button
                                    type="button"
                                    on:click=close_modal
                                    class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-fuchsia-500"
                                >
                                    Cancel
                                </button>
                                <button
                                    type="submit"
                                    class="px-4 py-2 text-sm font-medium text-white bg-gradient-to-r from-purple-600 via-fuchsia-600 to-indigo-600 border border-transparent rounded-lg hover:from-purple-700 hover:via-fuchsia-700 hover:to-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-fuchsia-500"
                                >
                                    Create Task
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            </Show>
        </main>
    }
}
