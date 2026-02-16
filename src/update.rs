use crate::db::get_image_blob;
use crate::github::{sync_orchids_to_github, upload_image_to_github};
use crate::model::{Cmd, Model, Msg};
use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Pure update function: applies a message to the model and returns side-effect commands.
/// This function contains NO side effects — it only mutates the model and declares intent.
pub fn update(model: &mut Model, msg: Msg) -> Vec<Cmd> {
    match msg {
        Msg::AddOrchid(orchid) => {
            model.orchids.push(orchid);
            vec![Cmd::Persist, Cmd::SyncToGitHub(model.orchids.clone())]
        }
        Msg::UpdateOrchid(orchid) => {
            if let Some(pos) = model.orchids.iter().position(|o| o.id == orchid.id) {
                model.orchids[pos] = orchid;
            }
            vec![Cmd::Persist, Cmd::SyncToGitHub(model.orchids.clone())]
        }
        Msg::DeleteOrchid(id) => {
            model.orchids.retain(|o| o.id != id);
            if model.selected_orchid.as_ref().is_some_and(|o| o.id == id) {
                model.selected_orchid = None;
            }
            vec![Cmd::Persist, Cmd::SyncToGitHub(model.orchids.clone())]
        }
        Msg::SetOrchids(orchids) => {
            model.orchids = orchids;
            vec![Cmd::Persist]
        }
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
        Msg::TriggerSync => {
            model.sync_status = "Syncing...".into();
            vec![Cmd::SyncToGitHub(model.orchids.clone())]
        }
        Msg::SetSyncStatus(status) => {
            let is_success = status == "Synced!";
            model.sync_status = status;
            if is_success {
                vec![Cmd::ClearSyncAfterDelay]
            } else {
                vec![]
            }
        }
        Msg::ClearSyncStatus => {
            if model.sync_status == "Synced!" {
                model.sync_status = String::new();
            }
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
        execute_cmd(cmd, set_model, model);
    }
}

/// Execute a single side-effect command. Async commands dispatch new messages when complete.
fn execute_cmd(cmd: Cmd, set_model: WriteSignal<Model>, model: ReadSignal<Model>) {
    match cmd {
        Cmd::Persist => {
            let m = model.get_untracked();
            let _ = LocalStorage::set("orchids", &m.orchids);
        }
        Cmd::SyncToGitHub(orchids) => {
            spawn_local(async move {
                let mut updated_orchids = orchids;
                let mut changes_made = false;

                for orchid in updated_orchids.iter_mut() {
                    for entry in orchid.history.iter_mut() {
                        if let Some(ref data) = entry.image_data {
                            if data.chars().all(char::is_numeric) {
                                if let Ok(id) = data.parse::<u32>() {
                                    if let Ok(Some(blob)) = get_image_blob(id).await {
                                        let promise = blob.array_buffer();
                                        if let Ok(ab) =
                                            wasm_bindgen_futures::JsFuture::from(promise).await
                                        {
                                            let uint8 = js_sys::Uint8Array::new(&ab);
                                            let vec = uint8.to_vec();
                                            let filename =
                                                format!("{}_{}.jpg", orchid.id, entry.id);

                                            match upload_image_to_github(filename, vec).await {
                                                Ok(remote_path) => {
                                                    entry.image_data = Some(remote_path);
                                                    changes_made = true;
                                                }
                                                Err(e) => log::error!(
                                                    "Failed to sync pending image: {}",
                                                    e
                                                ),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if changes_made {
                    dispatch(set_model, model, Msg::SetOrchids(updated_orchids.clone()));
                }

                match sync_orchids_to_github(updated_orchids).await {
                    Ok(_) => {
                        dispatch(set_model, model, Msg::SetSyncStatus("Synced!".into()));
                    }
                    Err(e) => {
                        log::error!("Sync failed: {}", e);
                        dispatch(
                            set_model,
                            model,
                            Msg::SetSyncStatus(format!("Error: {}", e)),
                        );
                    }
                }
            });
        }
        Cmd::ClearSyncAfterDelay => {
            spawn_local(async move {
                gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await;
                dispatch(set_model, model, Msg::ClearSyncStatus);
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ViewMode;
    use crate::orchid::{LightRequirement, Orchid, Placement};

    fn test_orchid(id: u64) -> Orchid {
        Orchid {
            id,
            name: format!("Test {}", id),
            species: "Test Species".into(),
            water_frequency_days: 7,
            light_requirement: LightRequirement::Medium,
            notes: String::new(),
            placement: Placement::Medium,
            light_lux: String::new(),
            temperature_range: String::new(),
            conservation_status: None,
            history: Vec::new(),
        }
    }

    #[test]
    fn test_add_orchid() {
        let mut model = Model::default();
        let orchid = test_orchid(1);
        let cmds = update(&mut model, Msg::AddOrchid(orchid));

        assert_eq!(model.orchids.len(), 1);
        assert_eq!(model.orchids[0].id, 1);
        assert!(cmds.iter().any(|c| matches!(c, Cmd::Persist)));
        assert!(cmds.iter().any(|c| matches!(c, Cmd::SyncToGitHub(_))));
    }

    #[test]
    fn test_update_orchid() {
        let mut model = Model {
            orchids: vec![test_orchid(1)],
            ..Default::default()
        };
        let mut updated = test_orchid(1);
        updated.name = "Updated Name".into();

        let cmds = update(&mut model, Msg::UpdateOrchid(updated));

        assert_eq!(model.orchids[0].name, "Updated Name");
        assert!(cmds.iter().any(|c| matches!(c, Cmd::Persist)));
        assert!(cmds.iter().any(|c| matches!(c, Cmd::SyncToGitHub(_))));
    }

    #[test]
    fn test_delete_orchid() {
        let mut model = Model {
            orchids: vec![test_orchid(1), test_orchid(2)],
            selected_orchid: Some(test_orchid(1)),
            ..Default::default()
        };

        let cmds = update(&mut model, Msg::DeleteOrchid(1));

        assert_eq!(model.orchids.len(), 1);
        assert_eq!(model.orchids[0].id, 2);
        assert!(model.selected_orchid.is_none());
        assert!(cmds.iter().any(|c| matches!(c, Cmd::Persist)));
    }

    #[test]
    fn test_delete_orchid_preserves_unrelated_selection() {
        let mut model = Model {
            orchids: vec![test_orchid(1), test_orchid(2)],
            selected_orchid: Some(test_orchid(2)),
            ..Default::default()
        };

        update(&mut model, Msg::DeleteOrchid(1));

        assert_eq!(model.selected_orchid.as_ref().unwrap().id, 2);
    }

    #[test]
    fn test_select_orchid() {
        let mut model = Model::default();
        let orchid = test_orchid(1);

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
    fn test_trigger_sync() {
        let mut model = Model {
            orchids: vec![test_orchid(1)],
            ..Default::default()
        };

        let cmds = update(&mut model, Msg::TriggerSync);

        assert_eq!(model.sync_status, "Syncing...");
        assert!(cmds.iter().any(|c| matches!(c, Cmd::SyncToGitHub(_))));
    }

    #[test]
    fn test_sync_success_triggers_clear_delay() {
        let mut model = Model::default();

        let cmds = update(&mut model, Msg::SetSyncStatus("Synced!".into()));

        assert_eq!(model.sync_status, "Synced!");
        assert!(cmds.iter().any(|c| matches!(c, Cmd::ClearSyncAfterDelay)));
    }

    #[test]
    fn test_sync_error_no_clear_delay() {
        let mut model = Model::default();

        let cmds = update(&mut model, Msg::SetSyncStatus("Error: failed".into()));

        assert_eq!(model.sync_status, "Error: failed");
        assert!(cmds.is_empty());
    }

    #[test]
    fn test_clear_sync_status() {
        let mut model = Model {
            sync_status: "Synced!".into(),
            ..Default::default()
        };

        update(&mut model, Msg::ClearSyncStatus);
        assert_eq!(model.sync_status, "");
    }

    #[test]
    fn test_clear_sync_status_preserves_error() {
        let mut model = Model {
            sync_status: "Error: something".into(),
            ..Default::default()
        };

        update(&mut model, Msg::ClearSyncStatus);
        assert_eq!(model.sync_status, "Error: something");
    }

    #[test]
    fn test_set_orchids() {
        let mut model = Model::default();
        let orchids = vec![test_orchid(1), test_orchid(2)];

        let cmds = update(&mut model, Msg::SetOrchids(orchids));

        assert_eq!(model.orchids.len(), 2);
        assert!(cmds.iter().any(|c| matches!(c, Cmd::Persist)));
        assert!(!cmds.iter().any(|c| matches!(c, Cmd::SyncToGitHub(_))));
    }
}
