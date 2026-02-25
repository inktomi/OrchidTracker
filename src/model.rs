use crate::components::scanner::AnalysisResult;
use crate::orchid::{GrowingZone, Orchid};

/// UI view mode toggle
#[derive(Clone, Debug, PartialEq)]
pub enum ViewMode {
    /// Displays items in a visual grid layout.
    Grid,
    /// Displays items in a detailed tabular format.
    Table,
}

/// Home page tab selection
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HomeTab {
    /// The primary tab displaying the user's plant collection.
    MyPlants,
    /// The tab displaying seasonal care information and transitions.
    Seasons,
}

/// Centralized UI state (TEA Model) — data now comes from server Resources
#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    /// The currently active view layout for the plant list.
    pub view_mode: ViewMode,
    /// The orchid currently selected for detailed viewing or editing.
    pub selected_orchid: Option<Orchid>,
    /// Whether the application settings modal is currently visible.
    pub show_settings: bool,
    /// Whether the AI plant scanner modal is currently active.
    pub show_scanner: bool,
    /// Whether the modal for adding a new orchid is open.
    pub show_add_modal: bool,
    /// Scanned data ready to be pre-filled into the add/edit form.
    pub prefill_data: Option<AnalysisResult>,
    /// The user's preferred temperature unit ("C" or "F").
    pub temp_unit: String,
    /// The user's hemisphere ("N" or "S") for seasonal calculations.
    pub hemisphere: String,
    /// Whether the dark visual theme is currently enabled.
    pub dark_mode: bool,
    /// The growing zone currently being configured in the setup wizard.
    pub wizard_zone: Option<GrowingZone>,
    /// The currently active tab on the home dashboard.
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
    /// Select an orchid to view details, or clear the selection.
    SelectOrchid(Option<Orchid>),
    /// Change the layout mode of the plant list.
    SetViewMode(ViewMode),

    // Modals
    /// Toggle the visibility of the settings modal.
    ShowSettings(bool),
    /// Toggle the visibility of the AI scanner modal.
    ShowScanner(bool),
    /// Toggle the visibility of the add orchid modal.
    ShowAddModal(bool),

    // Scanner
    /// Process the data returned from an AI scan.
    HandleScanResult(AnalysisResult),

    // Settings
    /// Triggered when the settings modal is closed, applying new preferences.
    SettingsClosed {
        /// The new temperature unit to apply.
        temp_unit: String,
    },

    // Theme
    /// Toggle between light and dark visual themes.
    ToggleDarkMode,

    // Wizard
    /// Open the setup wizard, optionally for a specific growing zone.
    ShowWizard(Option<GrowingZone>),

    // Home tab
    /// Change the active tab on the main dashboard.
    SetHomeTab(HomeTab),
}

/// Side effects returned by the update function (TEA Commands) — UI only
#[derive(Debug, PartialEq)]
pub enum Cmd {
    /// Command to apply the dark mode theme to the document body.
    ApplyDarkMode(bool),
}
