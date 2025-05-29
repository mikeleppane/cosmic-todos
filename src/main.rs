#![recursion_limit = "256"]

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> miette::Result<()> {
    use axum::Router;
    use cosmic_rust::app::App;
    use cosmic_rust::app::shell;
    use cosmic_rust::config::{get_config, initialize_config};
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};

    // Initialize configuration
    initialize_config()?;
    let app_config = get_config()
        .map_err(|e| miette::miette!("Failed to get configuration: {}", e))?
        .clone();

    let conf = get_configuration(None)
        .map_err(|e| miette::miette!("Failed to get Leptos configuration: {}", e))?;
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    leptos::logging::debug_warn!("Application configuration:\n {}", &app_config);
    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options)
        .with_state(app_config); // Inject app_config into state

    // Run our app with hyper
    leptos::logging::log!("🌌 Cosmic Todos listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| miette::miette!("Failed to bind to address {}: {}", addr, e))?;

    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|e| miette::miette!("Server error: {}", e))?;

    Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
