//! The main library module for OrchidTracker.
#![warn(missing_docs)]
#![recursion_limit = "512"]

#[allow(missing_docs)]
pub mod app;
#[allow(missing_docs)]
pub mod components;
/// Application error types and handling
pub mod error;
/// Climate and environment estimation algorithms
pub mod estimation;
/// Core domain models for orchids and related entities
pub mod orchid;
/// State management models following The Elm Architecture (TEA)
pub mod model;
/// State transition logic following The Elm Architecture (TEA)
pub mod update;
#[allow(missing_docs)]
pub mod pages;
/// Server functions for RPC calls between frontend and backend
pub mod server_fns;
/// Climate-aware dynamic watering algorithm
pub mod watering;

#[cfg(test)]
/// Helper functions and utilities for tests
pub mod test_helpers;

#[cfg(feature = "ssr")]
/// Database connection and repository implementations
pub mod db;
#[cfg(feature = "ssr")]
/// Authentication and authorization logic
pub mod auth;
#[cfg(feature = "ssr")]
/// Command Line Interface (CLI) configuration and handling
pub mod cli;
#[cfg(feature = "ssr")]
/// Climate sensor integration and data processing
pub mod climate;
#[cfg(feature = "ssr")]
/// Server configuration loading and management
pub mod config;
#[cfg(feature = "ssr")]
/// Cryptographic utilities (e.g. hashing)
pub mod crypto;
#[cfg(feature = "ssr")]
/// Push notification delivery mechanisms
pub mod push;
#[cfg(feature = "ssr")]
/// Management of user sessions
pub mod session_store;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    leptos::mount::hydrate_body(app::App);
}
