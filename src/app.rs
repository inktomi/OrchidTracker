use leptos::prelude::*;
use leptos::task::spawn_local;
use gloo_storage::{LocalStorage, Storage};
use crate::orchid::Orchid;
use crate::components::orchid_detail::OrchidDetail;
use crate::components::settings::SettingsModal;
use crate::components::scanner::{ScannerModal, AnalysisResult};
use crate::components::climate_dashboard::ClimateDashboard;
use crate::components::cabinet_table::OrchidCabinetTable;
use crate::components::orchid_card::OrchidCard;
use crate::components::add_orchid_form::AddOrchidForm;
use crate::github::{sync_orchids_to_github, upload_image_to_github};
use crate::db::get_image_blob;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq)]
enum ViewMode {
    Grid,
    Table,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClimateData {
    pub name: String,
    pub type_str: Option<String>,
    pub temperature: f64,
    pub humidity: f64,
    pub vpd: f64,
    pub updated: String,
}

#[component]
pub fn App() -> impl IntoView {
    let (orchids, set_orchids) = signal(
        LocalStorage::get("orchids").unwrap_or_else(|_| {
             let initial_data = include_str!("data/orchids.json");
             serde_json::from_str(initial_data).unwrap_or_else(|_| Vec::<Orchid>::new())
        })
    );

    let climate_data: Vec<ClimateData> = serde_json::from_str(include_str!("data/climate.json"))
        .unwrap_or_else(|_| Vec::new());
    let climate_data = StoredValue::new(climate_data);

    let (view_mode, set_view_mode) = signal(ViewMode::Grid);
    let (selected_orchid, set_selected_orchid) = signal::<Option<Orchid>>(None);

    // Check URL for deep link ID on load
    Effect::new(move |_| {
        if let Some(window) = web_sys::window() {
            if let Ok(search) = window.location().search() {
                if let Ok(params) = web_sys::UrlSearchParams::new_with_str(&search) {
                    if let Some(id_str) = params.get("id") {
                        if let Ok(id) = id_str.parse::<u64>() {
                            let current = orchids.get_untracked();
                            if let Some(o) = current.iter().find(|o| o.id == id) {
                                set_selected_orchid.set(Some(o.clone()));
                            }
                        }
                    }
                }
            }
        }
    });

    let (show_settings, set_show_settings) = signal(false);
    let (show_scanner, set_show_scanner) = signal(false);
    let (show_add_modal, set_show_add_modal) = signal(false);
    let (prefill_data, set_prefill_data) = signal::<Option<AnalysisResult>>(None);

    let (temp_unit, set_temp_unit) = signal("C".to_string());
    if let Ok(u) = LocalStorage::get("temp_unit") {
        set_temp_unit.set(u);
    }

    let (sync_status, set_sync_status) = signal(String::new());

    // Persist orchids to LocalStorage
    Effect::new(move |_| {
        let current_orchids = orchids.get();
        if let Err(e) = LocalStorage::set("orchids", &current_orchids) {
            log::error!("Failed to save to local storage: {:?}", e);
        }
    });

    let trigger_sync = move |orchids_vec: Vec<Orchid>| {
        set_sync_status.set("Syncing...".into());
        spawn_local(async move {
            let mut updated_orchids = orchids_vec.clone();
            let mut changes_made = false;

            for orchid in updated_orchids.iter_mut() {
                for entry in orchid.history.iter_mut() {
                    if let Some(ref data) = entry.image_data {
                        if data.chars().all(char::is_numeric) {
                            if let Ok(id) = data.parse::<u32>() {
                                if let Ok(Some(blob)) = get_image_blob(id).await {
                                    let promise = blob.array_buffer();
                                    if let Ok(ab) = wasm_bindgen_futures::JsFuture::from(promise).await {
                                        let uint8 = js_sys::Uint8Array::new(&ab);
                                        let vec = uint8.to_vec();
                                        let filename = format!("{}_{}.jpg", orchid.id, entry.id);

                                        match upload_image_to_github(filename, vec).await {
                                            Ok(remote_path) => {
                                                entry.image_data = Some(remote_path);
                                                changes_made = true;
                                            },
                                            Err(e) => log::error!("Failed to sync pending image: {}", e),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if changes_made {
                set_orchids.set(updated_orchids.clone());
            }

            match sync_orchids_to_github(updated_orchids).await {
                Ok(_) => {
                    set_sync_status.set("Synced!".into());
                    gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await;
                    set_sync_status.update(|s| if s == "Synced!" { *s = String::new(); });
                },
                Err(e) => {
                    log::error!("Sync failed: {}", e);
                    set_sync_status.set(format!("Error: {}", e));
                }
            }
        });
    };

    let add_orchid = move |new_orchid: Orchid| {
        set_orchids.update(|orchids| orchids.push(new_orchid.clone()));
        trigger_sync(orchids.get());
    };

    let update_orchid = move |updated_orchid: Orchid| {
        set_orchids.update(|orchids| {
            if let Some(pos) = orchids.iter().position(|o| o.id == updated_orchid.id) {
                orchids[pos] = updated_orchid;
            }
        });
        trigger_sync(orchids.get());
    };

    let delete_orchid = move |id: u64| {
        if let Some(window) = web_sys::window() {
            let confirmed = window.confirm_with_message("Are you sure you want to delete this orchid?").unwrap_or(false);
            if !confirmed {
                return;
            }
        }
        set_orchids.update(|orchids| {
            orchids.retain(|o| o.id != id);
        });
        if let Some(selected) = selected_orchid.get() {
            if selected.id == id {
                set_selected_orchid.set(None);
            }
        }
        trigger_sync(orchids.get());
    };

    let handle_scan_result = move |result: AnalysisResult| {
        set_prefill_data.set(Some(result));
        set_show_scanner.set(false);
        set_show_add_modal.set(true);
    };

    view! {
        <header>
            <div class="header-top">
                <h1>"Orchid Tracker"</h1>
                <div class="header-controls">
                    <span class="sync-status">
                        {move || sync_status.get()}
                        {move || sync_status.get().starts_with("Error:").then(|| {
                            view! { <button class="close-err-btn" on:click=move |_| set_sync_status.set(String::new())>"X"</button> }
                        })}
                    </span>
                    <button class="action-btn" on:click=move |_| trigger_sync(orchids.get())>"Sync"</button>
                    <button class="action-btn" on:click=move |_| set_show_add_modal.set(true)>"Add"</button>
                    <button class="action-btn" on:click=move |_| set_show_scanner.set(true)>"Scan"</button>
                    <button class="settings-btn" on:click=move |_| set_show_settings.set(true)>"Settings"</button>
                </div>
            </div>

            <ClimateDashboard data=climate_data unit=temp_unit />

            <div class="view-toggle">
                <button
                    class=move || if view_mode.get() == ViewMode::Grid { "active" } else { "" }
                    on:click=move |_| set_view_mode.set(ViewMode::Grid)
                >
                    "Grid View"
                </button>
                <button
                    class=move || if view_mode.get() == ViewMode::Table { "active" } else { "" }
                    on:click=move |_| set_view_mode.set(ViewMode::Table)
                >
                    "Placement View"
                </button>
            </div>
        </header>
        <main>
            {move || show_add_modal.get().then(|| {
                view! {
                    <AddOrchidForm
                        on_add=add_orchid
                        on_close=move || set_show_add_modal.set(false)
                        prefill_data=prefill_data
                    />
                }
            })}

            {move || match view_mode.get() {
                ViewMode::Grid => view! {
                    <div class="orchid-grid">
                        <For
                            each=move || orchids.get()
                            key=|orchid| orchid.id
                            children=move |orchid| {
                                let orchid_clone = orchid.clone();
                                view! {
                                    <OrchidCard
                                        orchid=orchid_clone
                                        on_delete=delete_orchid
                                        on_select=move |o| set_selected_orchid.set(Some(o))
                                    />
                                }
                            }
                        />
                    </div>
                }.into_any(),
                ViewMode::Table => view! {
                    <OrchidCabinetTable
                        orchids=orchids.get()
                        on_delete=delete_orchid
                        on_select=move |o| set_selected_orchid.set(Some(o))
                        on_update=update_orchid
                    />
                }.into_any()
            }}

            {move || selected_orchid.get().map(|orchid| {
                view! {
                    <OrchidDetail
                        orchid=orchid
                        on_close=move || set_selected_orchid.set(None)
                        on_update=update_orchid
                    />
                }
            })}

            {move || show_settings.get().then(|| {
                view! {
                    <SettingsModal on_close=move || {
                        set_show_settings.set(false);
                        if let Ok(u) = LocalStorage::get("temp_unit") {
                            set_temp_unit.set(u);
                        }
                    } />
                }
            })}

            {move || show_scanner.get().then(|| {
                let cd = climate_data.get_value();
                let summary = if !cd.is_empty() {
                    let d = &cd[0];
                    format!("Temp: {}C, Humidity: {}%, VPD: {}kPa", d.temperature, d.humidity, d.vpd)
                } else {
                    "Unknown climate".into()
                };

                view! {
                    <ScannerModal
                        on_close=move || set_show_scanner.set(false)
                        on_add_to_collection=handle_scan_result
                        existing_orchids=orchids.get()
                        climate_summary=summary
                    />
                }
            })}
        </main>
    }
}
