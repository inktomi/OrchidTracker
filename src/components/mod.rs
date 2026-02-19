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

// ── Shared UI Constants ──────────────────────────────────────────────

pub const MODAL_OVERLAY: &str = "fixed inset-0 flex justify-center items-center z-[1000] animate-fade-in bg-black/30 backdrop-blur-sm dark:bg-black/50";
pub const MODAL_CONTENT: &str = "bg-surface p-5 sm:p-8 rounded-2xl w-[95%] sm:w-[90%] max-w-[600px] max-h-[90vh] overflow-y-auto shadow-2xl animate-modal-in border border-stone-200/60 dark:border-stone-700/60";
pub const MODAL_HEADER: &str = "flex justify-between items-center mb-5 pb-4 border-b border-stone-200 dark:border-stone-700";

pub const BTN_PRIMARY: &str = "py-2.5 px-5 text-sm font-semibold text-white rounded-lg border-none cursor-pointer bg-primary hover:bg-primary-dark transition-colors";
pub const BTN_SECONDARY: &str = "py-2.5 px-5 text-sm font-semibold text-stone-600 bg-stone-100 rounded-lg border-none cursor-pointer hover:bg-stone-200 transition-colors dark:text-stone-300 dark:bg-stone-700 dark:hover:bg-stone-600";
pub const BTN_DANGER: &str = "py-1.5 px-3 text-xs font-semibold text-danger bg-danger/10 rounded-lg border-none cursor-pointer hover:bg-danger/20 transition-colors dark:text-red-300 dark:bg-red-900/30 dark:hover:bg-red-900/50";
pub const BTN_GHOST: &str = "py-2 px-3.5 text-sm font-medium text-white/90 bg-white/10 rounded-lg border border-white/20 cursor-pointer hover:bg-white/20 transition-colors";
pub const BTN_CLOSE: &str = "py-2 px-3 text-sm text-stone-400 bg-stone-100 rounded-lg border-none cursor-pointer hover:bg-stone-200 hover:text-stone-600 transition-colors dark:text-stone-400 dark:bg-stone-800 dark:hover:bg-stone-700 dark:hover:text-stone-200";
