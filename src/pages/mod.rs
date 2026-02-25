/// The main authenticated dashboard and primary entry point for logged-in users.
/// It exists to display the user's plant collection, alerts, and settings.
/// It is used by the router for the `/` path when a user session exists.
pub mod home;
/// The authentication screen for existing users to log into their account.
/// It exists to verify user credentials and establish a secure session.
/// It is used by the router for the `/login` path.
pub mod login;
/// The guided setup experience for brand new users.
/// It exists to collect initial data (like growing zones and first plants) smoothly.
/// It is used by the router for the `/welcome` path after registration.
pub mod onboarding;
/// The read-only view of a user's plant collection accessible to unauthenticated visitors.
/// It exists to allow users to share their collection via a public URL.
/// It is used by the router for the `/collection/:username` path.
pub mod public_collection;
/// The account creation screen for new users.
/// It exists to securely collect a new username, email, and password.
/// It is used by the router for the `/register` path.
pub mod register;
