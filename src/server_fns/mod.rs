#![warn(missing_docs)]
//! Server functions for the OrchidTracker application.

/// Server functions for managing and retrieving alerts.
pub mod alerts;
/// Server functions for user authentication and session management.
pub mod auth;
/// Server functions for managing climate data and sensor readings.
pub mod climate;
/// Server functions for integrating with IoT devices.
pub mod devices;
/// Server functions for managing orchid data and collections.
pub mod orchids;
/// Server functions for handling images.
pub mod images;
/// Server functions for managing user preferences.
pub mod preferences;
/// Server functions for interacting with AI scanning features.
pub mod scanner;
/// Server functions for handling public data access.
pub mod public;
/// Server functions for managing physical zones.
pub mod zones;
