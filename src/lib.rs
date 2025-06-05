pub mod api;
pub mod app_tmp;
pub mod auth;
pub mod components;
pub mod config;
pub mod config_tmp;
pub mod domain;
pub mod pages;
pub mod services;
pub mod todo;
pub mod utils;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app_tmp::*;
    use console_error_panic_hook;

    console_error_panic_hook::set_once();

    leptos::mount::hydrate_body(App);
}
