pub mod device_management;
pub mod orchid_detail;
pub mod settings;
pub mod scanner;
pub mod climate_dashboard;
pub mod cabinet_table;
pub mod orchid_card;
pub mod add_orchid_form;
pub mod app_header;
pub mod orchid_collection;
pub mod botanical_art;
pub mod habitat_weather;
pub mod notification_setup;
pub mod event_types;
pub mod event_type_picker;
pub mod quick_actions;
pub mod photo_capture;
pub mod growth_thread;
pub mod first_bloom;
pub mod photo_gallery;
pub mod seasonal_calendar;
pub mod zone_wizard;
pub mod manual_reading;
pub mod climate_strip;

// ── Shared UI Constants ──────────────────────────────────────────────

pub const MODAL_OVERLAY: &str = "fixed inset-0 flex justify-center items-center z-[1000] animate-fade-in bg-black/30 backdrop-blur-sm dark:bg-black/50";
pub const MODAL_CONTENT: &str = "bg-surface p-5 sm:p-8 rounded-2xl w-[95%] sm:w-[90%] max-w-[600px] max-h-[90vh] overflow-y-auto shadow-2xl animate-modal-in border border-stone-200/60 dark:border-stone-700/60";
pub const MODAL_HEADER: &str = "flex justify-between items-center mb-5 pb-4 border-b border-stone-200 dark:border-stone-700";

pub const BTN_PRIMARY: &str = "py-2.5 px-5 text-sm font-semibold text-white rounded-lg border-none cursor-pointer bg-primary hover:bg-primary-dark transition-colors";
pub const BTN_SECONDARY: &str = "py-2.5 px-5 text-sm font-semibold text-stone-600 bg-stone-100 rounded-lg border-none cursor-pointer hover:bg-stone-200 transition-colors dark:text-stone-300 dark:bg-stone-700 dark:hover:bg-stone-600";
pub const BTN_DANGER: &str = "py-1.5 px-3 text-xs font-semibold text-danger bg-danger/10 rounded-lg border-none cursor-pointer hover:bg-danger/20 transition-colors dark:text-red-300 dark:bg-red-900/30 dark:hover:bg-red-900/50";
pub const BTN_GHOST: &str = "py-2 px-3.5 text-sm font-medium text-white/90 bg-white/10 rounded-lg border border-white/20 cursor-pointer hover:bg-white/20 transition-colors";
pub const BTN_CLOSE: &str = "py-2 px-3 text-sm text-stone-400 bg-stone-100 rounded-lg border-none cursor-pointer hover:bg-stone-200 hover:text-stone-600 transition-colors dark:text-stone-400 dark:bg-stone-800 dark:hover:bg-stone-700 dark:hover:text-stone-200";

// ── Shared Climate Helpers ───────────────────────────────────────────
use leptos::prelude::*;

pub const BADGE_ESTIMATED: &str = "inline-flex gap-1 items-center py-0.5 px-2.5 text-[10px] font-bold tracking-wide rounded-full bg-amber-100/80 text-amber-700 dark:bg-amber-900/30 dark:text-amber-300";
pub const BADGE_MANUAL: &str = "inline-flex gap-1 items-center py-0.5 px-2.5 text-[10px] font-bold tracking-wide rounded-full bg-sky-100/80 text-sky-700 dark:bg-sky-900/30 dark:text-sky-300";
pub const BADGE_LIVE: &str = "inline-flex gap-1 items-center py-0.5 px-2.5 text-[10px] font-bold tracking-wide rounded-full bg-emerald-100/80 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300";

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
