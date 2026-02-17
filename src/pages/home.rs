use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::components::add_orchid_form::AddOrchidForm;
use crate::components::cabinet_table::OrchidCabinetTable;
use crate::components::climate_dashboard::ClimateDashboard;
use crate::components::orchid_card::OrchidCard;
use crate::components::orchid_detail::OrchidDetail;
use crate::components::scanner::ScannerModal;
use crate::components::settings::SettingsModal;
use crate::components::BTN_GHOST;
use crate::model::ViewMode;
use crate::orchid::Orchid;
use crate::server_fns::auth::get_current_user;
use crate::server_fns::orchids::{get_orchids, create_orchid, update_orchid, delete_orchid};

const TAB_ACTIVE: &str = "py-2 px-4 text-sm font-semibold text-primary bg-surface rounded-lg shadow-sm cursor-pointer border-none transition-all dark:text-primary-light";
const TAB_INACTIVE: &str = "py-2 px-4 text-sm font-medium text-stone-500 bg-transparent rounded-lg cursor-pointer border-none transition-all hover:text-stone-700 dark:text-stone-400 dark:hover:text-stone-200";

#[component]
pub fn HomePage() -> impl IntoView {
    let navigate = use_navigate();

    // Check auth — redirect to login if not authenticated
    let user = Resource::new(|| (), |_| get_current_user());

    // Load orchids from server
    let orchids_resource = Resource::new(|| (), |_| get_orchids());

    // UI state (client-only, not server data)
    let (view_mode, set_view_mode) = signal(ViewMode::Grid);
    let (selected_orchid, set_selected_orchid) = signal::<Option<Orchid>>(None);
    let (show_settings, set_show_settings) = signal(false);
    let (show_scanner, set_show_scanner) = signal(false);
    let (show_add_modal, set_show_add_modal) = signal(false);
    let (prefill_data, set_prefill_data) = signal::<Option<crate::components::scanner::AnalysisResult>>(None);
    let (temp_unit, _set_temp_unit) = signal("C".to_string());
    let (dark_mode, set_dark_mode) = signal(false);

    // Static climate data (included at compile time)
    let climate_data: Vec<crate::app::ClimateData> =
        serde_json::from_str(include_str!("../data/climate.json")).unwrap_or_default();
    let climate_data = StoredValue::new(climate_data);

    let temp_unit_memo = Memo::new(move |_| temp_unit.get());
    let prefill_memo = Memo::new(move |_| prefill_data.get());

    // Dark mode toggle
    let toggle_dark_mode = move |_| {
        let new_val = !dark_mode.get();
        set_dark_mode.set(new_val);
        #[cfg(feature = "hydrate")]
        {
            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                if let Some(root) = document.document_element() {
                    let class_list = root.class_list();
                    if new_val {
                        let _ = class_list.add_1("dark");
                    } else {
                        let _ = class_list.remove_1("dark");
                    }
                }
            }
        }
    };

    // Orchid operations via server functions
    let on_add = move |orchid: Orchid| {
        leptos::task::spawn_local(async move {
            let _ = create_orchid(
                orchid.name,
                orchid.species,
                orchid.water_frequency_days,
                orchid.light_requirement.to_string(),
                orchid.notes,
                orchid.placement.to_string(),
                orchid.light_lux,
                orchid.temperature_range,
                orchid.conservation_status,
            ).await;
            orchids_resource.refetch();
        });
    };

    let on_update = move |orchid: Orchid| {
        leptos::task::spawn_local(async move {
            let _ = update_orchid(orchid.clone()).await;
            orchids_resource.refetch();
        });
    };

    let on_delete = move |id: String| {
        #[cfg(feature = "hydrate")]
        {
            if let Some(window) = web_sys::window() {
                if !window.confirm_with_message("Are you sure you want to delete this orchid?").unwrap_or(false) {
                    return;
                }
            }
        }
        leptos::task::spawn_local(async move {
            let _ = delete_orchid(id).await;
            orchids_resource.refetch();
        });
    };

    let on_select = move |o: Orchid| {
        set_selected_orchid.set(Some(o));
    };

    let on_settings_close = move || {
        set_show_settings.set(false);
    };

    let on_scan_result = move |result: crate::components::scanner::AnalysisResult| {
        set_prefill_data.set(Some(result));
        set_show_scanner.set(false);
        set_show_add_modal.set(true);
    };

    view! {
        <Suspense fallback=move || view! { <p class="p-8 text-center text-stone-500">"Loading..."</p> }>
            {move || {
                let nav = navigate.clone();
                user.get().map(|result| match result {
                    Ok(Some(_user_info)) => view! { <div></div> }.into_any(),
                    _ => {
                        // Not authenticated, redirect
                        nav("/login", Default::default());
                        view! { <div></div> }.into_any()
                    }
                })
            }}
        </Suspense>

        <header class="bg-primary">
            <div class="flex flex-wrap gap-3 justify-between items-center py-3 px-4 mx-auto sm:px-6 max-w-[1200px]">
                <h1 class="m-0 text-xl tracking-wide text-white">"Orchid Tracker"</h1>
                <div class="flex flex-wrap gap-2 items-center">
                    <button class=BTN_GHOST on:click=toggle_dark_mode>
                        {move || if dark_mode.get() { "\u{2600}" } else { "\u{263E}" }}
                    </button>
                    <button class=BTN_GHOST on:click=move |_| set_show_add_modal.set(true)>"Add"</button>
                    <button class=BTN_GHOST on:click=move |_| set_show_scanner.set(true)>"Scan"</button>
                    <button class=BTN_GHOST on:click=move |_| set_show_settings.set(true)>"Settings"</button>
                </div>
            </div>
        </header>

        <main class="py-6 px-4 mx-auto sm:px-6 max-w-[1200px]">
            <ClimateDashboard data=climate_data unit=temp_unit_memo />

            <div class="flex justify-center mb-6">
                <div class="inline-flex gap-1 p-1 rounded-xl bg-secondary">
                    <button
                        class=move || if view_mode.get() == ViewMode::Grid { TAB_ACTIVE } else { TAB_INACTIVE }
                        on:click=move |_| set_view_mode.set(ViewMode::Grid)
                    >
                        "Grid View"
                    </button>
                    <button
                        class=move || if view_mode.get() == ViewMode::Table { TAB_ACTIVE } else { TAB_INACTIVE }
                        on:click=move |_| set_view_mode.set(ViewMode::Table)
                    >
                        "Shelf View"
                    </button>
                </div>
            </div>

            {move || show_add_modal.get().then(|| {
                view! {
                    <AddOrchidForm
                        on_add=on_add
                        on_close=move || set_show_add_modal.set(false)
                        prefill_data=prefill_memo
                    />
                }
            })}

            <Suspense fallback=move || view! { <p class="text-center text-stone-500">"Loading orchids..."</p> }>
                {move || orchids_resource.get().map(|result| {
                    let orchids = result.unwrap_or_default();
                    let orchids_for_table = orchids.clone();
                    match view_mode.get() {
                        ViewMode::Grid => view! {
                            <div class="grid gap-5 grid-cols-[repeat(auto-fill,minmax(300px,1fr))]">
                                <For
                                    each=move || orchids.clone()
                                    key=|orchid| orchid.id.clone()
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
                                orchids=orchids_for_table
                                on_delete=on_delete
                                on_select=on_select
                                on_update=on_update
                            />
                        }.into_any()
                    }
                })}
            </Suspense>

            {move || selected_orchid.get().map(|orchid| {
                view! {
                    <OrchidDetail
                        orchid=orchid
                        on_close=move || set_selected_orchid.set(None)
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
                let orchids = orchids_resource.get().and_then(|r| r.ok()).unwrap_or_default();
                view! {
                    <ScannerModal
                        on_close=move || set_show_scanner.set(false)
                        on_add_to_collection=on_scan_result
                        existing_orchids=orchids
                        climate_data=climate_data.get_value()
                    />
                }
            })}
        </main>
    }
}
