//! The main library module for OrchidTracker.
#![warn(missing_docs)]
#![recursion_limit = "512"]

#[allow(missing_docs)]
pub mod app;
#[allow(missing_docs)]
pub mod components;

/// What is it? Application error types and handling.
/// Why does it exist? To provide a centralized definition of all ways the application can fail, allowing for structured error reporting.
/// How should it be used? Use the `AppError` enum throughout the codebase via `Result<T, AppError>` and map underlying errors into it.
pub mod error;

/// What is it? Climate and environment estimation algorithms.
/// Why does it exist? To provide logic for guessing local climate conditions when physical sensors are not available (e.g. from user wizard answers).
/// How should it be used? Call functions like `estimate_indoor_climate` within the onboarding wizard or zone configuration.
pub mod estimation;

/// What is it? Core domain models for orchids and related entities.
/// Why does it exist? To define the shape and constraints of the data fundamental to tracking an orchid collection.
/// How should it be used? Import structs like `Orchid`, `GrowingZone`, and `LogEntry` when manipulating data or sending it between client and server.
pub mod orchid;

/// What is it? State management models following The Elm Architecture (TEA).
/// Why does it exist? To consolidate all frontend UI state and allowed transitions into a single predictable flow.
/// How should it be used? Read from the `Model` in UI components and construct `Msg` enums to request state changes.
pub mod model;

/// What is it? State transition logic following The Elm Architecture (TEA).
/// Why does it exist? To encapsulate the pure logic of how the UI state changes in response to messages without directly coupling to the DOM.
/// How should it be used? Call `update::dispatch` from UI event handlers to push a new `Msg` into the system.
pub mod update;

#[allow(missing_docs)]
pub mod pages;

/// What is it? Server functions for RPC calls between frontend and backend.
/// Why does it exist? To define Leptos `#[server]` macros that transparently handle async API calls from the WebAssembly client to the Rust backend.
/// How should it be used? Call these functions directly from frontend code as if they were local async functions; they automatically serialize over HTTP.
pub mod server_fns;

/// What is it? Climate-aware dynamic watering algorithm.
/// Why does it exist? To calculate adaptive watering intervals based on real-time temperature, humidity, and species requirements.
/// How should it be used? Call `climate_adjusted_watering` before displaying watering countdowns in the UI or processing alerts.
pub mod watering;

#[cfg(test)]
/// What is it? Helper functions and utilities for tests.
/// Why does it exist? To provide shared mock data and setup routines for the test suite without compiling them into the production binary.
/// How should it be used? Import inside `#[cfg(test)]` modules to quickly scaffold test scenarios.
pub mod test_helpers;

#[cfg(feature = "ssr")]
/// What is it? Database connection and repository implementations.
/// Why does it exist? To manage the SurrealDB lifecycle, schema migrations, and low-level data access for the backend.
/// How should it be used? Call `init_db()` at server startup and use `db()` to acquire a connection handle for queries.
pub mod db;

#[cfg(feature = "ssr")]
/// What is it? Authentication and authorization logic.
/// Why does it exist? To securely handle passwords, session cookies, and user verification.
/// How should it be used? Use functions like `hash_password` and `verify_password` during login/registration, and `get_user_id` to protect endpoints.
pub mod auth;

#[cfg(feature = "ssr")]
/// What is it? Command Line Interface (CLI) configuration and handling.
/// Why does it exist? To allow the application to be started with custom arguments or run standalone maintenance commands (like database migrations).
/// How should it be used? Parse it in `main.rs` to determine the server's execution mode before starting the HTTP listener.
pub mod cli;

#[cfg(feature = "ssr")]
/// What is it? Climate sensor integration and data processing.
/// Why does it exist? To encapsulate the logic for polling third-party hardware APIs (like Tempest or AC Infinity) and storing their data.
/// How should it be used? Spawn the poller tasks from this module in the background during server initialization.
pub mod climate;

#[cfg(feature = "ssr")]
/// What is it? Server configuration loading and management.
/// Why does it exist? To read environment variables and provide a strongly-typed configuration struct for the backend.
/// How should it be used? Call `init_config()` at startup and access values via the global configuration instance.
pub mod config;

#[cfg(feature = "ssr")]
/// What is it? Cryptographic utilities (e.g. hashing).
/// Why does it exist? To provide specific low-level crypto operations, such as VAPID key generation or secure tokens.
/// How should it be used? Call these functions when generating secure push notification payloads or resetting passwords.
pub mod crypto;

#[cfg(feature = "ssr")]
/// What is it? Push notification delivery mechanisms.
/// Why does it exist? To handle the Web Push protocol and dispatch alerts to subscribed user devices.
/// How should it be used? Call functions in this module from background tasks when an alert condition is met.
pub mod push;

#[cfg(feature = "ssr")]
/// What is it? Management of user sessions.
/// Why does it exist? To store and retrieve active session data (like the logged-in user ID) from SurrealDB via the `tower-sessions` crate.
/// How should it be used? Attach it as a layer to the Axum router so sessions are automatically managed per HTTP request.
pub mod session_store;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
/// What is it? Main entry point for the WebAssembly frontend.
/// Why does it exist? To connect the server-rendered HTML payload to the client-side Leptos reactivity system.
/// How should it be used? It is automatically invoked by the generated JavaScript bindings when the WASM module loads in the browser.
pub fn hydrate() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    leptos::mount::hydrate_body(app::App);
}
