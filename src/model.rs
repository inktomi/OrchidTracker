use crate::components::scanner::AnalysisResult;
use crate::orchid::Orchid;

/// UI view mode toggle
#[derive(Clone, Debug, PartialEq)]
pub enum ViewMode {
    Grid,
    Table,
}

/// Centralized UI state (TEA Model) — data now comes from server Resources
#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub view_mode: ViewMode,
    pub selected_orchid: Option<Orchid>,
    pub show_settings: bool,
    pub show_scanner: bool,
    pub show_add_modal: bool,
    pub prefill_data: Option<AnalysisResult>,
    pub temp_unit: String,
    pub dark_mode: bool,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            view_mode: ViewMode::Grid,
            selected_orchid: None,
            show_settings: false,
            show_scanner: false,
            show_add_modal: false,
            prefill_data: None,
            temp_unit: "C".to_string(),
            dark_mode: false,
        }
    }
}

/// All possible state transitions (TEA Messages) — UI state only
pub enum Msg {
    // Navigation
    SelectOrchid(Option<Orchid>),
    SetViewMode(ViewMode),

    // Modals
    ShowSettings(bool),
    ShowScanner(bool),
    ShowAddModal(bool),

    // Scanner
    HandleScanResult(AnalysisResult),

    // Settings
    SettingsClosed { temp_unit: String },

    // Theme
    ToggleDarkMode,
}

/// Side effects returned by the update function (TEA Commands) — UI only
#[derive(Debug, PartialEq)]
pub enum Cmd {
    ApplyDarkMode(bool),
}
