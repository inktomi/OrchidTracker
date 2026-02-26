use crate::components::scanner::AnalysisResult;
use crate::orchid::{GrowingZone, Orchid};

/// What is it? A toggle representing the layout style for the primary plant list.
/// Why does it exist? It allows the user to switch between a visual grid of cards and a denser tabular data view.
/// How should it be used? Read from `Model::view_mode` to determine which component to render, and dispatch `Msg::SetViewMode` to change it.
#[derive(Clone, Debug, PartialEq)]
pub enum ViewMode {
    /// Displays items in a visual grid layout.
    Grid,
    /// Displays items in a detailed tabular format.
    Table,
}

/// What is it? A selection representing the active tab on the main dashboard.
/// Why does it exist? It separates the user's personal collection view from global seasonal care information.
/// How should it be used? Read from `Model::home_tab` to display the correct tab content, and dispatch `Msg::SetHomeTab` when a user clicks a tab button.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HomeTab {
    /// The primary tab displaying the user's plant collection.
    MyPlants,
    /// The tab displaying tasks that need to be done today.
    Tasks,
    /// The tab displaying seasonal care information and transitions.
    Seasons,
}

/// What is it? The central state struct for the application's UI, following The Elm Architecture (TEA).
/// Why does it exist? It consolidates all client-side UI state into a single source of truth, making state transitions predictable and testable.
/// How should it be used? Store it in a Leptos signal at the root of the application, derive fine-grained `Memo`s for component props, and mutate it exclusively through the `update` function via `Msg` dispatches.
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

/// What is it? An enumeration of all possible state transitions in the application's UI.
/// Why does it exist? It acts as the single mechanism for triggering state changes, ensuring all updates flow synchronously through a pure function.
/// How should it be used? Construct a specific variant in response to a user action (e.g., clicking a button) and pass it to the `update::dispatch` function to modify the `Model`.
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

/// What is it? An enumeration of side-effects that the application needs to perform after a state update.
/// Why does it exist? It keeps the core `update` function pure by returning declarative descriptions of asynchronous or browser-specific actions (like changing themes).
/// How should it be used? Return variants of this enum from the `update` function, which will then be interpreted and executed by the `execute_cmd` function.
#[derive(Debug, PartialEq)]
pub enum Cmd {
    /// Command to apply the dark mode theme to the document body.
    ApplyDarkMode(bool),
}
