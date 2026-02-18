#![recursion_limit = "512"]

pub mod app;
pub mod components;
pub mod error;
pub mod orchid;
pub mod model;
pub mod update;
pub mod pages;
pub mod server_fns;

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
pub mod session_store;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
}
