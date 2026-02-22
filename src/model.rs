use crate::components::scanner::AnalysisResult;
use crate::orchid::{GrowingZone, Orchid};

/// UI view mode toggle
#[derive(Clone, Debug, PartialEq)]
pub enum ViewMode {
    Grid,
    Table,
}

/// Home page tab selection
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HomeTab {
    MyPlants,
    Seasons,
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
    pub hemisphere: String,
    pub dark_mode: bool,
    pub wizard_zone: Option<GrowingZone>,
    pub home_tab: HomeTab,
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
            hemisphere: "N".to_string(),
            dark_mode: false,
            wizard_zone: None,
            home_tab: HomeTab::MyPlants,
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

    // Wizard
    ShowWizard(Option<GrowingZone>),

    // Home tab
    SetHomeTab(HomeTab),
}

/// Side effects returned by the update function (TEA Commands) — UI only
#[derive(Debug, PartialEq)]
pub enum Cmd {
    ApplyDarkMode(bool),
}
