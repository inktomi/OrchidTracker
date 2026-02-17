use crate::components::scanner::AnalysisResult;
use crate::orchid::Orchid;
use gloo_storage::{LocalStorage, Storage};

/// UI view mode toggle
#[derive(Clone, Debug, PartialEq)]
pub enum ViewMode {
    Grid,
    Table,
}

/// Centralized application state (TEA Model)
#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub orchids: Vec<Orchid>,
    pub view_mode: ViewMode,
    pub selected_orchid: Option<Orchid>,
    pub show_settings: bool,
    pub show_scanner: bool,
    pub show_add_modal: bool,
    pub prefill_data: Option<AnalysisResult>,
    pub temp_unit: String,
    pub sync_status: String,
    pub dark_mode: bool,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            orchids: Vec::new(),
            view_mode: ViewMode::Grid,
            selected_orchid: None,
            show_settings: false,
            show_scanner: false,
            show_add_modal: false,
            prefill_data: None,
            temp_unit: "C".to_string(),
            sync_status: String::new(),
            dark_mode: false,
        }
    }
}

impl Model {
    /// Initialize model from browser storage and URL params (WASM only)
    pub fn init() -> Self {
        let orchids = LocalStorage::get("orchids").unwrap_or_else(|_| {
            let initial_data = include_str!("data/orchids.json");
            serde_json::from_str(initial_data).unwrap_or_else(|_| Vec::<Orchid>::new())
        });

        let temp_unit = LocalStorage::get("temp_unit").unwrap_or_else(|_| "C".to_string());
        let dark_mode = LocalStorage::get("dark_mode").unwrap_or(false);
        let selected_orchid = Self::check_deep_link(&orchids);

        Self {
            orchids,
            temp_unit,
            dark_mode,
            selected_orchid,
            ..Default::default()
        }
    }

    fn check_deep_link(orchids: &[Orchid]) -> Option<Orchid> {
        let window = web_sys::window()?;
        let search = window.location().search().ok()?;
        let params = web_sys::UrlSearchParams::new_with_str(&search).ok()?;
        let id_str = params.get("id")?;
        let id = id_str.parse::<u64>().ok()?;
        orchids.iter().find(|o| o.id == id).cloned()
    }
}

/// All possible state transitions (TEA Messages)
pub enum Msg {
    // Orchid CRUD
    AddOrchid(Orchid),
    UpdateOrchid(Orchid),
    DeleteOrchid(u64),
    SetOrchids(Vec<Orchid>),

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

    // Sync
    TriggerSync,
    SetSyncStatus(String),
    ClearSyncStatus,
}

/// Side effects returned by the update function (TEA Commands)
#[derive(Debug, PartialEq)]
pub enum Cmd {
    Persist,
    SyncToGitHub(Vec<Orchid>),
    ClearSyncAfterDelay,
    ApplyDarkMode(bool),
}
