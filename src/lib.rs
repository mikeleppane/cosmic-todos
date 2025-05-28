pub mod app;
pub mod auth;
pub mod config;
pub mod services;
pub mod todo;
pub mod views;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    use console_error_panic_hook;

    console_error_panic_hook::set_once();

    leptos::mount::hydrate_body(App);
}
