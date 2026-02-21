use crate::model::{Cmd, Model, Msg};
use leptos::prelude::*;

/// Pure update function: applies a message to the model and returns side-effect commands.
/// This function contains NO side effects — it only mutates the model and declares intent.
pub fn update(model: &mut Model, msg: Msg) -> Vec<Cmd> {
    match msg {
        Msg::SelectOrchid(orchid) => {
            model.selected_orchid = orchid;
            vec![]
        }
        Msg::SetViewMode(mode) => {
            model.view_mode = mode;
            vec![]
        }
        Msg::ShowSettings(show) => {
            model.show_settings = show;
            vec![]
        }
        Msg::ShowScanner(show) => {
            model.show_scanner = show;
            vec![]
        }
        Msg::ShowAddModal(show) => {
            model.show_add_modal = show;
            vec![]
        }
        Msg::HandleScanResult(result) => {
            model.prefill_data = Some(result);
            model.show_scanner = false;
            model.show_add_modal = true;
            vec![]
        }
        Msg::SettingsClosed { temp_unit } => {
            model.show_settings = false;
            model.temp_unit = temp_unit;
            vec![]
        }
        Msg::ToggleDarkMode => {
            model.dark_mode = !model.dark_mode;
            vec![Cmd::ApplyDarkMode(model.dark_mode)]
        }
        Msg::ShowWizard(zone) => {
            model.wizard_zone = zone;
            vec![]
        }
    }
}

/// Dispatch a message: update the model, then execute any resulting commands.
pub fn dispatch(set_model: WriteSignal<Model>, model: ReadSignal<Model>, msg: Msg) {
    let mut m = model.get_untracked();
    let cmds = update(&mut m, msg);
    set_model.set(m);
    for cmd in cmds {
        execute_cmd(cmd);
    }
}

/// Execute a single side-effect command.
fn execute_cmd(cmd: Cmd) {
    match cmd {
        Cmd::ApplyDarkMode(enabled) => {
            #[cfg(feature = "hydrate")]
            {
                if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                    if let Some(root) = document.document_element() {
                        let class_list = root.class_list();
                        if enabled {
                            let _ = class_list.add_1("dark");
                        } else {
                            let _ = class_list.remove_1("dark");
                        }
                    }
                }
            }
            let _ = enabled; // suppress unused warning in SSR
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ViewMode;
    use crate::orchid::{LightRequirement, Orchid};

    fn test_orchid(id: &str) -> Orchid {
        Orchid {
            id: id.to_string(),
            name: format!("Test {}", id),
            species: "Test Species".into(),
            water_frequency_days: 7,
            light_requirement: LightRequirement::Medium,
            notes: String::new(),
            placement: "Medium Light Area".to_string(),
            light_lux: String::new(),
            temperature_range: String::new(),
            conservation_status: None,
            native_region: None,
            native_latitude: None,
            native_longitude: None,
            last_watered_at: None,
            temp_min: None,
            temp_max: None,
            humidity_min: None,
            humidity_max: None,
            first_bloom_at: None,
            last_fertilized_at: None,
            fertilize_frequency_days: None,
            fertilizer_type: None,
            last_repotted_at: None,
            pot_medium: None,
            pot_size: None,
            rest_start_month: None,
            rest_end_month: None,
            bloom_start_month: None,
            bloom_end_month: None,
            rest_water_multiplier: None,
            rest_fertilizer_multiplier: None,
            active_water_multiplier: None,
            active_fertilizer_multiplier: None,
        }
    }

    #[test]
    fn test_select_orchid() {
        let mut model = Model::default();
        let orchid = test_orchid("1");

        let cmds = update(&mut model, Msg::SelectOrchid(Some(orchid.clone())));

        assert_eq!(model.selected_orchid, Some(orchid));
        assert!(cmds.is_empty());
    }

    #[test]
    fn test_set_view_mode() {
        let mut model = Model::default();
        assert_eq!(model.view_mode, ViewMode::Grid);

        let cmds = update(&mut model, Msg::SetViewMode(ViewMode::Table));

        assert_eq!(model.view_mode, ViewMode::Table);
        assert!(cmds.is_empty());
    }

    #[test]
    fn test_modal_toggles() {
        let mut model = Model::default();

        update(&mut model, Msg::ShowSettings(true));
        assert!(model.show_settings);

        update(&mut model, Msg::ShowScanner(true));
        assert!(model.show_scanner);

        update(&mut model, Msg::ShowAddModal(true));
        assert!(model.show_add_modal);
    }

    #[test]
    fn test_handle_scan_result_opens_add_modal() {
        let mut model = Model {
            show_scanner: true,
            ..Default::default()
        };

        let result = crate::components::scanner::AnalysisResult {
            species_name: "Phal".into(),
            fit_category: crate::orchid::FitCategory::GoodFit,
            reason: "Great fit".into(),
            already_owned: false,
            water_freq: 7,
            light_req: LightRequirement::Medium,
            temp_range: "20-30C".into(),
            placement_suggestion: "Medium".into(),
            conservation_status: None,
            native_region: None,
            native_latitude: None,
            native_longitude: None,
            temp_min: None,
            temp_max: None,
            humidity_min: None,
            humidity_max: None,
            rest_start_month: None,
            rest_end_month: None,
            bloom_start_month: None,
            bloom_end_month: None,
            rest_water_multiplier: None,
            rest_fertilizer_multiplier: None,
            active_water_multiplier: None,
            active_fertilizer_multiplier: None,
        };

        update(&mut model, Msg::HandleScanResult(result));

        assert!(!model.show_scanner);
        assert!(model.show_add_modal);
        assert!(model.prefill_data.is_some());
    }

    #[test]
    fn test_settings_closed() {
        let mut model = Model {
            show_settings: true,
            temp_unit: "C".into(),
            ..Default::default()
        };

        let cmds = update(
            &mut model,
            Msg::SettingsClosed {
                temp_unit: "F".into(),
            },
        );

        assert!(!model.show_settings);
        assert_eq!(model.temp_unit, "F");
        assert!(cmds.is_empty());
    }

    #[test]
    fn test_show_wizard() {
        let mut model = Model::default();
        assert!(model.wizard_zone.is_none());

        let zone = crate::orchid::GrowingZone {
            id: "gz:1".into(),
            name: "Test Zone".into(),
            light_level: LightRequirement::Medium,
            location_type: crate::orchid::LocationType::Indoor,
            temperature_range: String::new(),
            humidity: String::new(),
            description: String::new(),
            sort_order: 0,
            data_source_type: None,
            data_source_config: String::new(),
            hardware_device_id: None,
            hardware_port: None,
        };

        let cmds = update(&mut model, Msg::ShowWizard(Some(zone.clone())));
        assert_eq!(model.wizard_zone.as_ref().unwrap().id, "gz:1");
        assert!(cmds.is_empty());

        let cmds = update(&mut model, Msg::ShowWizard(None));
        assert!(model.wizard_zone.is_none());
        assert!(cmds.is_empty());
    }

    #[test]
    fn test_toggle_dark_mode() {
        let mut model = Model::default();
        assert!(!model.dark_mode);

        let cmds = update(&mut model, Msg::ToggleDarkMode);
        assert!(model.dark_mode);
        assert!(cmds.iter().any(|c| matches!(c, Cmd::ApplyDarkMode(true))));

        let cmds = update(&mut model, Msg::ToggleDarkMode);
        assert!(!model.dark_mode);
        assert!(cmds.iter().any(|c| matches!(c, Cmd::ApplyDarkMode(false))));
    }
}
