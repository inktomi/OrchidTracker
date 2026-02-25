#![recursion_limit = "512"]

pub mod app;
pub mod components;
pub mod error;
pub mod estimation;
pub mod orchid;
pub mod model;
pub mod update;
pub mod pages;
pub mod server_fns;
pub mod watering;

#[cfg(test)]
pub mod test_helpers;

#[cfg(feature = "ssr")]
pub mod db;
#[cfg(feature = "ssr")]
pub mod auth;
#[cfg(feature = "ssr")]
pub mod cli;
#[cfg(feature = "ssr")]
pub mod climate;
#[cfg(feature = "ssr")]
pub mod config;
#[cfg(feature = "ssr")]
pub mod crypto;
#[cfg(feature = "ssr")]
pub mod push;
#[cfg(feature = "ssr")]
pub mod session_store;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    leptos::mount::hydrate_body(app::App);
}
