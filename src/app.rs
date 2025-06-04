#![allow(clippy::must_use_candidate)]
use leptos::{leptos_dom::logging, prelude::*};
use leptos_meta::{Link, MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

use crate::{
    auth::{AuthProvider, use_auth},
    todo::Todo,
    views::{home::HomePage, login::LoginPage},
};

// Static configuration loaded once at startup

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
    //let (authenticated, set_authenticated) = signal(false);

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/cosmic-rust.css" />

        // sets the document title
        <Title text="Family Leppänen Todos" />

        <Link
            rel="icon"
            type_="image/png"
            sizes="64x64"
            href="/images/familyleppanen-logo-64x64.png"
        />

        // content for this welcome page
        <AuthProvider>
            <AppRoutes />
        </AuthProvider>
    }
}

#[component]
fn AppRoutes() -> impl IntoView {
    let auth = use_auth();

    logging::console_log("AppRoutes initialized, checking authentication...");
    logging::console_log(&format!(
        "Rendering HomePage, authenticated: {}",
        auth.is_authenticated.get(),
    ));

    view! {
        <Show
            when=move || !auth.is_loading.get()
            fallback=move || {
                view! {
                    <div class="flex items-center justify-center min-h-screen">
                        <div class="text-lg">Loading...</div>
                    </div>
                }
            }
        >
            <Router>
                <main>
                    <Routes fallback=|| "Page not found.">
                        <Route
                            path=StaticSegment("")
                            view=move || {
                                logging::console_log(
                                    &format!(
                                        "Rendering HomePage, authenticated: {}",
                                        auth.is_authenticated.get(),
                                    ),
                                );
                                if auth.is_authenticated.get() {
                                    view! { <HomePage /> }.into_any()
                                } else {
                                    view! { <LoginPage /> }.into_any()
                                }
                            }
                        />
                        <Route path=StaticSegment("login") view=move || view! { <LoginPage /> } />
                        <Route
                            path=StaticSegment("todo")
                            view=move || {
                                logging::console_log(
                                    &format!(
                                        "Rendering HomePage, authenticated: {}",
                                        auth.is_authenticated.get(),
                                    ),
                                );
                                if auth.is_authenticated.get() {
                                    view! { <HomePage /> }.into_any()
                                } else {
                                    view! { <LoginPage /> }.into_any()
                                }
                            }
                        />
                    </Routes>
                </main>
            </Router>
        </Show>
    }
}

/// Redirects to a different page
/* #[component]
fn Redirect(path: &'static str) -> impl IntoView {
    let navigate = use_navigate();

    Effect::new(move |_| {
        navigate(path, NavigateOptions::default());
    });
} */
// Server functions for Cosmos DB operations
#[server(CreateTodo, "/api")]
pub async fn create_todo_server(todo: Todo) -> Result<Todo, ServerFnError> {
    use crate::services::cosmos_service::get_cosmos_service;
    use leptos::logging;
    use validator::Validate;

    // Validate input
    todo.validate()
        .map_err(|e| ServerFnError::new(format!("Validation error: {}", e)))?;

    // Sanitize strings
    #[allow(unused_variables)]
    let sanitized_todo = Todo {
        title: sanitize_string(&todo.title),
        description: todo.description.map(|desc| sanitize_string(&desc)),
        ..todo
    };

    // Initialize DB on first access
    logging::log!("Initializing Cosmos DB...");
    let cosmos_service = get_cosmos_service()
        .map_err(|e| ServerFnError::new(format!("Failed to get Cosmos service: {e}")))?;

    logging::log!("Creating todo in Cosmos DB: {:?}", sanitized_todo);

    let cosmos_todo = cosmos_service
        .create_todo(sanitized_todo)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create todo: {e}")))?;

    logging::log!("Created todo in Cosmos DB: {:?}", cosmos_todo);

    Ok(Todo::from(cosmos_todo))
}

#[server(name=GetTodos, prefix="/api")]
pub async fn get_todos_server() -> Result<Vec<Todo>, ServerFnError> {
    use crate::services::cosmos_service::get_cosmos_service;
    use leptos::logging;

    let cosmos_service = get_cosmos_service()
        .map_err(|e| ServerFnError::new(format!("Failed to get Cosmos service: {e}")))?;

    logging::log!("Retrieving todos from Cosmos DB...");

    let cosmos_todos = cosmos_service
        .get_todos()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get todos: {e}")))?;

    let todos: Vec<Todo> = cosmos_todos.into_iter().map(Todo::from).collect();

    logging::log!("Retrieved {} todos from Cosmos DB", todos.len());

    Ok(todos)
}

#[server(UpdateTodo, "/api")]
pub async fn update_todo_server(todo: Todo) -> Result<Todo, ServerFnError> {
    use crate::services::cosmos_service::get_cosmos_service;
    use leptos::logging;

    let cosmos_service = get_cosmos_service()
        .map_err(|e| ServerFnError::new(format!("Failed to get Cosmos service: {e}")))?;

    let cosmos_todo = cosmos_service
        .update_todo(todo)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update todo: {e}")))?;

    logging::log!("Updated todo in Cosmos DB: {:?}", cosmos_todo);

    Ok(Todo::from(cosmos_todo))
}

#[server(DeleteTodo, "/api")]
pub async fn delete_todo_server(todo_id: String) -> Result<(), ServerFnError> {
    use crate::services::cosmos_service::get_cosmos_service;
    use leptos::logging;

    let cosmos_service = get_cosmos_service()
        .map_err(|e| ServerFnError::new(format!("Failed to get Cosmos service: {e}")))?;

    cosmos_service
        .delete_todo(&todo_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to delete todo: {e}")))?;

    logging::log!("Deleted todo from Cosmos DB: {todo_id}");

    Ok(())
}

#[allow(dead_code)]
fn sanitize_string(input: &str) -> String {
    // Remove potential HTML/script tags and normalize whitespace
    input
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .trim()
        .to_string()
}
