/// Component for managing connected climate sensors and hardware devices.
/// It exists to let users configure physical hardware integrations (e.g., Tempest, AC Infinity).
/// It is used within the settings modal when configuring zones.
pub mod device_management;
/// Modal component displaying detailed information and history for a single orchid.
/// It exists to provide a deep dive into an orchid's timeline without leaving the main view.
/// It is used when a user clicks on an orchid card in the collection grid.
pub mod orchid_detail;
/// Modal component for user preferences, application settings, and device configuration.
/// It exists to give users a central place to control their app experience (themes, units, notifications).
/// It is used by opening it from the main application header or user menu.
pub mod settings;
/// Component integrating AI vision and text analysis for plant identification.
/// It exists to help users identify orchids or parse care instructions from images and text.
/// It is used either when adding a new plant or scanning a tag from the main dashboard.
pub mod scanner;
/// Dashboard widget showing real-time and historical climate data for a zone.
/// It exists to give users a quick overview of the environmental conditions affecting their orchids.
/// It is used on the main home screen or within a zone's detailed view.
pub mod climate_dashboard;
/// Interactive table view for managing orchid placements and zones.
/// It exists to allow drag-and-drop reassignment of orchids between different physical locations.
/// It is used as an alternative view mode to the standard collection grid.
pub mod cabinet_table;
/// Individual card component representing a single orchid in the collection grid.
/// It exists to summarize key information (photo, name, watering status) at a glance.
/// It is used iteratively within the `orchid_collection` component.
pub mod orchid_card;
/// Form component for adding a new orchid to the user's collection.
/// It exists to collect necessary data (name, species, placement) to track a new plant.
/// It is used within a modal triggered by the "Add Plant" button.
pub mod add_orchid_form;
/// Top-level navigation bar for the application.
/// It exists to provide persistent access to main actions, settings, and user profile.
/// It is used at the top of the main layout across all primary views.
pub mod app_header;
/// Container component that renders the grid of `orchid_card`s.
/// It exists to manage the layout and filtering of the user's entire plant collection.
/// It is used as the primary content area on the home page.
pub mod orchid_collection;
/// Component rendering botanical illustrations or decorative background elements.
/// It exists to enhance the aesthetic feel of the application, especially in empty states.
/// It is used in onboarding, empty collection views, or as a background layer.
pub mod botanical_art;
/// Widget displaying weather data from an orchid's native geographic region.
/// It exists to help users understand the natural conditions their plant evolved in.
/// It is used within the `orchid_detail` view alongside local climate data.
pub mod habitat_weather;
/// Component handling the browser prompt for push notification permissions.
/// It exists to abstract the complexities of subscribing to web push notifications.
/// It is used within the settings modal or as a banner prompt for new users.
pub mod notification_setup;
/// Definitions and constants for various timeline event types (watering, repotting, etc.).
/// It exists to provide a centralized registry of event metadata and visual styling.
/// It is used by the `orchid_detail` timeline and the `quick_actions` component.
pub mod event_types;
/// Interactive picker component for selecting a timeline event type.
/// It exists to give users a visual way to choose what kind of log entry they are creating.
/// It is used within the form for adding a new event to an orchid's timeline.
pub mod event_type_picker;
/// Row of fast-action buttons (water, fertilize, log) for common tasks.
/// It exists to reduce friction for the most frequent user interactions.
/// It is used on the `orchid_card` and at the top of the `orchid_detail` view.
pub mod quick_actions;
/// Component handling camera integration and image file selection.
/// It exists to allow users to attach photos to timeline events or scan plant tags.
/// It is used within the timeline entry form and the AI scanner modal.
pub mod photo_capture;
/// Visual timeline of an orchid's growth and care history.
/// It exists to present a chronological, scrollable record of events for a specific plant.
/// It is used as the primary content of the `orchid_detail` modal.
pub mod growth_thread;
/// Specialized component highlighting the first time an orchid blooms under a user's care.
/// It exists to celebrate a significant milestone in an orchid grower's journey.
/// It is used within the `growth_thread` or as a special badge on the `orchid_card`.
pub mod first_bloom;
/// Grid view of all photos attached to an orchid's timeline.
/// It exists to provide a purely visual browsing experience of a plant's history.
/// It is used as an alternate tab or view within the `orchid_detail` modal.
pub mod photo_gallery;
/// Calendar widget showing an orchid's natural rest and bloom cycles.
/// It exists to help users anticipate care changes based on the plant's seasonal needs.
/// It is used within the `orchid_detail` view and the seasonal dashboard tab.
pub mod seasonal_calendar;
/// Multi-step form for creating and configuring a new growing zone.
/// It exists to guide users through estimating indoor conditions or linking hardware sensors.
/// It is used during user onboarding and when adding new zones from the settings.
pub mod zone_wizard;
/// Dashboard for today's tasks.
/// It exists to show a list of plants that need to be watered today.
/// It is used as a tab on the home page.
pub mod today_tasks;
/// Inline form for manually entering current temperature and humidity.
/// It exists to allow users without automated sensors to record climate snapshots.
/// It is used within the `climate_dashboard` or zone settings.
pub mod manual_reading;
/// Compact horizontal banner displaying top-level climate alerts or summaries.
/// It exists to surface critical environmental issues without occupying much screen space.
/// It is used at the top of the home page or specific zone views.
pub mod climate_strip;
/// Cookie consent banner shown on first visit.
/// It exists to inform users about our essential session cookie per GDPR/CCPA.
/// It is rendered globally in the App component and dismisses after acknowledgment.
pub mod cookie_consent;

// ── Shared UI Constants ──────────────────────────────────────────────

/// CSS classes for a full-screen, semi-transparent modal overlay.
/// It exists to ensure a consistent visual backdrop for all modals across the app.
/// It is used in the `class` attribute of the outermost `div` of any modal component.
pub const MODAL_OVERLAY: &str = "fixed inset-0 flex justify-center items-center z-[1000] animate-fade-in bg-black/30 backdrop-blur-sm dark:bg-black/50";
/// CSS classes for the main content box of a modal window.
/// It exists to provide consistent sizing, padding, and animations for modal bodies.
/// It is used in the `class` attribute of the inner container of any modal component.
pub const MODAL_CONTENT: &str = "bg-surface p-5 sm:p-8 rounded-2xl w-[95%] sm:w-[90%] max-w-[600px] max-h-[90vh] overflow-y-auto shadow-2xl animate-modal-in border border-stone-200/60 dark:border-stone-700/60";
/// CSS classes for the header section of a modal (title and close button).
/// It exists to maintain a uniform look and spacing for modal headers.
/// It is used in the `class` attribute of the header `div` within a modal.
pub const MODAL_HEADER: &str = "flex justify-between items-center mb-5 pb-4 border-b border-stone-200 dark:border-stone-700";

/// CSS classes for a primary call-to-action button.
/// It exists to provide a distinct, high-contrast button for main actions (e.g., Save, Submit).
/// It is used in the `class` attribute of primary `<button>` elements.
pub const BTN_PRIMARY: &str = "py-2.5 px-5 text-sm font-semibold text-white rounded-lg border-none cursor-pointer bg-primary hover:bg-primary-dark transition-colors";
/// CSS classes for a secondary, lower-prominence button.
/// It exists to provide an alternative button style for less critical actions (e.g., Cancel, Edit).
/// It is used in the `class` attribute of secondary `<button>` elements.
pub const BTN_SECONDARY: &str = "py-2.5 px-5 text-sm font-semibold text-stone-600 bg-stone-100 rounded-lg border-none cursor-pointer hover:bg-stone-200 transition-colors dark:text-stone-300 dark:bg-stone-700 dark:hover:bg-stone-600";
/// CSS classes for a destructive or warning button.
/// It exists to visually warn the user that the action is irreversible (e.g., Delete).
/// It is used in the `class` attribute of dangerous `<button>` elements.
pub const BTN_DANGER: &str = "py-1.5 px-3 text-xs font-semibold text-danger bg-danger/10 rounded-lg border-none cursor-pointer hover:bg-danger/20 transition-colors dark:text-red-300 dark:bg-red-900/30 dark:hover:bg-red-900/50";
/// CSS classes for a translucent button suitable for dark backgrounds.
/// It exists to provide a button that blends well over imagery or dark gradients.
/// It is used in the `class` attribute of buttons placed on dark headers or photo galleries.
pub const BTN_GHOST: &str = "py-2 px-3.5 text-sm font-medium text-white/90 bg-white/10 rounded-lg border border-white/20 cursor-pointer hover:bg-white/20 transition-colors";
/// CSS classes specifically for modal close buttons (e.g., an 'X' icon).
/// It exists to provide a standard, recognizable dismiss action for modals.
/// It is used in the `class` attribute of the close button within `MODAL_HEADER`.
pub const BTN_CLOSE: &str = "py-2 px-3 text-sm text-stone-400 bg-stone-100 rounded-lg border-none cursor-pointer hover:bg-stone-200 hover:text-stone-600 transition-colors dark:text-stone-400 dark:bg-stone-800 dark:hover:bg-stone-700 dark:hover:text-stone-200";

// ── Shared Climate Helpers ───────────────────────────────────────────
use leptos::prelude::*;

/// CSS classes for an 'Estimated' data source badge.
/// It exists to visually indicate that climate data is derived from wizard estimates, not sensors.
/// It is used within `source_badge` for estimated data.
pub const BADGE_ESTIMATED: &str = "inline-flex gap-1 items-center py-0.5 px-2.5 text-[10px] font-bold tracking-wide rounded-full bg-amber-100/80 text-amber-700 dark:bg-amber-900/30 dark:text-amber-300";
/// CSS classes for a 'Manual' data source badge.
/// It exists to visually indicate that climate data was manually entered by the user.
/// It is used within `source_badge` for manual data.
pub const BADGE_MANUAL: &str = "inline-flex gap-1 items-center py-0.5 px-2.5 text-[10px] font-bold tracking-wide rounded-full bg-sky-100/80 text-sky-700 dark:bg-sky-900/30 dark:text-sky-300";
/// CSS classes for a 'Live' data source badge.
/// It exists to visually indicate that climate data is coming from a real-time hardware sensor.
/// It is used within `source_badge` for live sensor data.
pub const BADGE_LIVE: &str = "inline-flex gap-1 items-center py-0.5 px-2.5 text-[10px] font-bold tracking-wide rounded-full bg-emerald-100/80 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300";

/// Generates a visual Leptos UI badge indicating the provenance of climate data.
/// It exists to quickly inform the user how reliable or recent a given climate reading is.
/// It is used in views that display climate readings, like `climate_dashboard` or `climate_strip`.
pub fn source_badge(source: &Option<String>) -> Option<leptos::tachys::view::any_view::AnyView> {
    match source.as_deref() {
        Some("wizard") => Some(leptos::IntoView::into_view(
            leptos::view! { <span class=BADGE_ESTIMATED>"Estimated"</span> }
        ).into_any()),
        Some("manual") => Some(leptos::IntoView::into_view(
            leptos::view! { <span class=BADGE_MANUAL>"Manual"</span> }
        ).into_any()),
        Some(s) if !s.is_empty() => Some(leptos::IntoView::into_view(
            leptos::view! { <span class=BADGE_LIVE>"Live"</span> }
        ).into_any()),
        _ => None,
    }
}

/// Formats a UTC timestamp into a human-readable relative string (e.g., "5 min ago").
/// It exists to provide a more intuitive understanding of data freshness than absolute timestamps.
/// It is used in UI components that display recent events or sensor readings.
pub fn format_time_ago(recorded_at: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(*recorded_at);

    if diff.num_minutes() < 1 {
        "just now".to_string()
    } else if diff.num_minutes() < 60 {
        format!("{} min ago", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{} hr ago", diff.num_hours())
    } else {
        format!("{} days ago", diff.num_days())
    }
}
pub mod suitability_card;
