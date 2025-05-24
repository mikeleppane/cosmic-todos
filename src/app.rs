#![allow(clippy::must_use_candidate)]
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    NavigateOptions, StaticSegment,
    components::{Route, Router, Routes},
    hooks::use_navigate,
};

use crate::views::{home::HomePage, login::LoginPage};

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
