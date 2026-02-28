#![warn(missing_docs)]
//! **What is it?**
//! The root module for all Leptos Server Functions in the OrchidTracker application.
//!
//! **Why does it exist?**
//! It organizes the RPC layer that bridges the frontend and backend, enabling seamless cross-network calls.
//!
//! **How should it be used?**
//! Frontend components should import and call the exposed `#[server]` functions from these submodules to interact with the backend database and external APIs.

/// **What is it?**
/// A module containing server functions for retrieving and managing system alerts.
///
/// **Why does it exist?**
/// It exists to provide the frontend with access to real-time and historical alert data, such as climate warnings.
///
/// **How should it be used?**
/// Call these functions from UI components that need to display or acknowledge alerts.
pub mod alerts;
/// **What is it?**
/// A module containing server functions for user authentication and session management.
///
/// **Why does it exist?**
/// It exists to securely handle user registration, login, logout, and session validation across the network boundary.
///
/// **How should it be used?**
/// Call these functions from authentication forms or middleware to verify user identity and manage sessions.
pub mod auth;
/// **What is it?**
/// A module containing server functions for managing climate data and sensor readings.
///
/// **Why does it exist?**
/// It exists to allow the frontend to fetch current and historical environmental data for habitats and zones.
///
/// **How should it be used?**
/// Call these functions from dashboard components or charts to display temperature, humidity, and other climate metrics.
pub mod climate;
/// **What is it?**
/// A module containing server functions for integrating with IoT devices.
///
/// **Why does it exist?**
/// It exists to manage the connection, configuration, and status of external hardware like smart plugs or sensors.
///
/// **How should it be used?**
/// Call these functions from device management UI views to register new devices or change their settings.
pub mod devices;
/// **What is it?**
/// A module containing server functions for managing orchid data and collections.
///
/// **Why does it exist?**
/// It is the core data module for CRUD operations on a user's orchid collection, allowing them to add, edit, or remove plants.
///
/// **How should it be used?**
/// Call these functions from views like the main collection grid or detail pages to load and mutate orchid records.
pub mod orchids;
/// **What is it?**
/// A module containing server functions for handling image uploads and retrieval.
///
/// **Why does it exist?**
/// It exists to provide a dedicated channel for securely uploading, processing, and accessing image binaries associated with orchids.
///
/// **How should it be used?**
/// Call these functions from image upload forms or when rendering image galleries for specific orchids.
pub mod images;
/// **What is it?**
/// A module containing server functions for managing user preferences.
///
/// **Why does it exist?**
/// It exists to persist user-specific settings, such as preferred temperature units (C/F) or UI themes.
///
/// **How should it be used?**
/// Call these functions from a settings page or to hydrate user context upon application load.
pub mod preferences;
/// **What is it?**
/// A module containing server functions for interacting with AI scanning features.
///
/// **Why does it exist?**
/// It exists to offload the heavy processing of image recognition and AI-driven data extraction to the backend.
///
/// **How should it be used?**
/// Call these functions when submitting an image from the scanner UI to identify an orchid or diagnose a problem.
pub mod scanner;
/// **What is it?**
/// A module containing server functions for handling public data access.
///
/// **Why does it exist?**
/// It exists to expose non-authenticated routes or shared data that anyone visiting the application can view.
///
/// **How should it be used?**
/// Call these functions from landing pages or public gallery views where no user session is required.
pub mod public;
/// **What is it?**
/// A module providing a client-side telemetry proxy to Axiom.
///
/// **Why does it exist?**
/// It exists to forward structured log events from the WASM client to Axiom via a server function,
/// since the browser cannot send traces to Axiom directly without exposing the API token.
///
/// **How should it be used?**
/// Call `telemetry::emit_info/emit_warn/emit_error` from client-side code to send structured events to Axiom.
pub mod telemetry;
/// **What is it?**
/// A module containing server functions for managing physical zones.
///
/// **Why does it exist?**
/// It exists to organize orchids into distinct physical locations (like a greenhouse or a specific window).
///
/// **How should it be used?**
/// Call these functions from zone management views to create, edit, or delete the locations where plants are kept.
pub mod zones;
