use crate::components::add_orchid_form::AddOrchidForm;
use crate::components::cabinet_table::OrchidCabinetTable;
use crate::components::climate_dashboard::ClimateDashboard;
use crate::components::orchid_card::OrchidCard;
use crate::components::orchid_detail::OrchidDetail;
use crate::components::scanner::ScannerModal;
use crate::components::settings::SettingsModal;
use crate::components::BTN_GHOST;
use crate::model::{Model, Msg, ViewMode};
use crate::update;
use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClimateData {
    pub name: String,
    pub type_str: Option<String>,
    pub temperature: f64,
    pub humidity: f64,
    pub vpd: f64,
    pub updated: String,
}

const TAB_ACTIVE: &str = "py-2 px-4 text-sm font-semibold text-primary bg-surface rounded-lg shadow-sm cursor-pointer border-none transition-all";
const TAB_INACTIVE: &str = "py-2 px-4 text-sm font-medium text-stone-500 bg-transparent rounded-lg cursor-pointer border-none hover:text-stone-700 transition-all";

#[component]
pub fn App() -> impl IntoView {
    // ── Model ──────────────────────────────────────────────────────────
    let (model, set_model) = signal(Model::init());

    // Static data (not part of TEA model — never changes)
    let climate_data: Vec<ClimateData> =
        serde_json::from_str(include_str!("data/climate.json")).unwrap_or_else(|_| Vec::new());
    let climate_data = StoredValue::new(climate_data);

    // ── Selectors (Memos for fine-grained reactivity) ─────────────────
    let orchids = Memo::new(move |_| model.get().orchids.clone());
    let view_mode = Memo::new(move |_| model.get().view_mode.clone());
    let sync_status = Memo::new(move |_| model.get().sync_status.clone());
    let selected_orchid = Memo::new(move |_| model.get().selected_orchid.clone());
    let show_add_modal = Memo::new(move |_| model.get().show_add_modal);
    let show_settings = Memo::new(move |_| model.get().show_settings);
    let show_scanner = Memo::new(move |_| model.get().show_scanner);
    let temp_unit = Memo::new(move |_| model.get().temp_unit.clone());
    let prefill_data = Memo::new(move |_| model.get().prefill_data.clone());

    // ── Persist orchids to LocalStorage on every model change ─────────
    Effect::new(move |_| {
        let current_orchids = orchids.get();
        if let Err(e) = LocalStorage::set("orchids", &current_orchids) {
            log::error!("Failed to save to local storage: {:?}", e);
        }
    });

    // ── Update: thin callback wrappers that dispatch messages ─────────
    let on_add = move |orchid| update::dispatch(set_model, model, Msg::AddOrchid(orchid));
    let on_update = move |orchid| update::dispatch(set_model, model, Msg::UpdateOrchid(orchid));
    let on_select = move |o| update::dispatch(set_model, model, Msg::SelectOrchid(Some(o)));

    let on_delete = move |id: u64| {
        if let Some(window) = web_sys::window() {
            if !window
                .confirm_with_message("Are you sure you want to delete this orchid?")
                .unwrap_or(false)
            {
                return;
            }
        }
        update::dispatch(set_model, model, Msg::DeleteOrchid(id));
    };

    let on_settings_close = move || {
        let unit = LocalStorage::get("temp_unit").unwrap_or_else(|_| "C".to_string());
        update::dispatch(set_model, model, Msg::SettingsClosed { temp_unit: unit });
    };

    let on_scan_result =
        move |result| update::dispatch(set_model, model, Msg::HandleScanResult(result));

    // ── View ──────────────────────────────────────────────────────────
    view! {
        <header class="bg-primary">
            <div class="flex flex-wrap gap-3 justify-between items-center py-3 px-4 mx-auto sm:px-6 max-w-[1200px]">
                <h1 class="m-0 text-xl tracking-wide text-white">"Orchid Tracker"</h1>
                <div class="flex flex-wrap gap-2 items-center">
                    {move || {
                        let status = sync_status.get();
                        (!status.is_empty()).then(|| {
                            let has_error = status.starts_with("Error:");
                            view! {
                                <span class="py-1 px-2.5 text-xs font-medium rounded-md text-white/80 bg-white/15">
                                    {status}
                                    {has_error.then(|| {
                                        view! { <button class="p-0 ml-1.5 text-xs bg-transparent border-none cursor-pointer hover:text-white text-white/60" on:click=move |_| update::dispatch(set_model, model, Msg::SetSyncStatus(String::new()))>"×"</button> }
                                    })}
                                </span>
                            }
                        })
                    }}
                    <button class=BTN_GHOST on:click=move |_| update::dispatch(set_model, model, Msg::TriggerSync)>"Sync"</button>
                    <button class=BTN_GHOST on:click=move |_| update::dispatch(set_model, model, Msg::ShowAddModal(true))>"Add"</button>
                    <button class=BTN_GHOST on:click=move |_| update::dispatch(set_model, model, Msg::ShowScanner(true))>"Scan"</button>
                    <button class=BTN_GHOST on:click=move |_| update::dispatch(set_model, model, Msg::ShowSettings(true))>"Settings"</button>
                </div>
            </div>
        </header>
        <main class="py-6 px-4 mx-auto sm:px-6 max-w-[1200px]">
            <ClimateDashboard data=climate_data unit=temp_unit />

            <div class="flex justify-center mb-6">
                <div class="inline-flex gap-1 p-1 rounded-xl bg-secondary">
                    <button
                        class=move || if view_mode.get() == ViewMode::Grid { TAB_ACTIVE } else { TAB_INACTIVE }
                        on:click=move |_| update::dispatch(set_model, model, Msg::SetViewMode(ViewMode::Grid))
                    >
                        "Grid View"
                    </button>
                    <button
                        class=move || if view_mode.get() == ViewMode::Table { TAB_ACTIVE } else { TAB_INACTIVE }
                        on:click=move |_| update::dispatch(set_model, model, Msg::SetViewMode(ViewMode::Table))
                    >
                        "Shelf View"
                    </button>
                </div>
            </div>

            {move || show_add_modal.get().then(|| {
                view! {
                    <AddOrchidForm
                        on_add=on_add
                        on_close=move || update::dispatch(set_model, model, Msg::ShowAddModal(false))
                        prefill_data=prefill_data
                    />
                }
            })}

            {move || match view_mode.get() {
                ViewMode::Grid => view! {
                    <div class="grid gap-5 grid-cols-[repeat(auto-fill,minmax(300px,1fr))]">
                        <For
                            each=move || orchids.get()
                            key=|orchid| orchid.id
                            children=move |orchid| {
                                let orchid_clone = orchid.clone();
                                view! {
                                    <OrchidCard
                                        orchid=orchid_clone
                                        on_delete=on_delete
                                        on_select=on_select
                                    />
                                }
                            }
                        />
                    </div>
                }.into_any(),
                ViewMode::Table => view! {
                    <OrchidCabinetTable
                        orchids=orchids.get()
                        on_delete=on_delete
                        on_select=on_select
                        on_update=on_update
                    />
                }.into_any()
            }}

            {move || selected_orchid.get().map(|orchid| {
                view! {
                    <OrchidDetail
                        orchid=orchid
                        on_close=move || update::dispatch(set_model, model, Msg::SelectOrchid(None))
                        on_update=on_update
                    />
                }
            })}

            {move || show_settings.get().then(|| {
                view! {
                    <SettingsModal on_close=on_settings_close />
                }
            })}

            {move || show_scanner.get().then(|| {
                view! {
                    <ScannerModal
                        on_close=move || update::dispatch(set_model, model, Msg::ShowScanner(false))
                        on_add_to_collection=on_scan_result
                        existing_orchids=orchids.get()
                        climate_data=climate_data.get_value()
                    />
                }
            })}
        </main>
    }
}
