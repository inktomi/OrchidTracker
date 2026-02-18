use leptos::prelude::*;
use crate::components::add_orchid_form::AddOrchidForm;
use crate::components::app_header::AppHeader;
use crate::components::botanical_art::OrchidAccent;
use crate::components::climate_dashboard::ClimateDashboard;
use crate::components::orchid_collection::OrchidCollection;
use crate::components::orchid_detail::OrchidDetail;
use crate::components::scanner::ScannerModal;
use crate::components::settings::SettingsModal;
use crate::model::{Model, Msg};
use crate::orchid::Orchid;
use crate::server_fns::auth::get_current_user;
use crate::server_fns::orchids::{get_orchids, create_orchid, update_orchid, delete_orchid};
use crate::server_fns::zones::{get_zones, migrate_legacy_placements};
use crate::update::dispatch;

#[component]
pub fn HomePage() -> impl IntoView {
    // Check auth — redirect to login if not authenticated
    let user = Resource::new(|| (), |_| get_current_user());

    // Load orchids from server
    let orchids_resource = Resource::new(|| (), |_| get_orchids());

    // Run legacy migration once on load, then load zones
    let migration_resource = Resource::new(|| (), |_| migrate_legacy_placements());
    let (zones_version, set_zones_version) = signal(0u32);
    let zones_resource = Resource::new(
        move || (migration_resource.get(), zones_version.get()),
        |_| get_zones(),
    );

    let zones_memo = Memo::new(move |_| {
        zones_resource.get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
    });

    // TEA model + dispatch
    let (model, set_model) = signal(Model::default());
    let send = move |msg: Msg| dispatch(set_model, model, msg);

    // Derived memos for fine-grained reactivity
    let view_mode = Memo::new(move |_| model.get().view_mode);
    let selected_orchid = Memo::new(move |_| model.get().selected_orchid.clone());
    let show_settings = Memo::new(move |_| model.get().show_settings);
    let show_scanner = Memo::new(move |_| model.get().show_scanner);
    let show_add_modal = Memo::new(move |_| model.get().show_add_modal);
    let prefill_data = Memo::new(move |_| model.get().prefill_data.clone());
    let temp_unit = Memo::new(move |_| model.get().temp_unit.clone());
    let dark_mode = Memo::new(move |_| model.get().dark_mode);

    // Dynamic climate readings from configured data sources
    let climate_resource = Resource::new(
        move || zones_version.get(),
        |_| crate::server_fns::climate::get_current_readings(),
    );

    let climate_readings = Memo::new(move |_| {
        climate_resource.get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
    });

    // Orchid operations via server functions (async I/O — not TEA state)
    let on_add = move |orchid: Orchid| {
        leptos::task::spawn_local(async move {
            let _ = create_orchid(
                orchid.name,
                orchid.species,
                orchid.water_frequency_days,
                orchid.light_requirement.to_string(),
                orchid.notes,
                orchid.placement.clone(),
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

    let on_zones_changed = move || {
        set_zones_version.update(|v| *v += 1);
    };

    view! {
        <Suspense fallback=move || view! { <p class="p-8 text-center text-stone-500">"Loading..."</p> }>
            {move || {
                user.get().map(|result| match result {
                    Ok(Some(_user_info)) => {
                        // Check if user needs onboarding (no zones)
                        let zones = zones_memo.get();
                        if zones.is_empty() {
                            // Only redirect after migration has completed and zones are actually loaded
                            if migration_resource.get().is_some() && zones_resource.get().is_some() {
                                #[cfg(feature = "ssr")]
                                leptos_axum::redirect("/onboarding");
                                #[cfg(feature = "hydrate")]
                                {
                                    if let Some(window) = web_sys::window() {
                                        let _ = window.location().set_href("/onboarding");
                                    }
                                }
                            }
                        }
                        view! { <div></div> }.into_any()
                    },
                    _ => {
                        #[cfg(feature = "ssr")]
                        leptos_axum::redirect("/login");
                        #[cfg(feature = "hydrate")]
                        {
                            if let Some(window) = web_sys::window() {
                                let _ = window.location().set_href("/login");
                            }
                        }
                        view! { <div></div> }.into_any()
                    }
                })
            }}
        </Suspense>

        <AppHeader
            dark_mode=dark_mode
            on_toggle_dark=move || send(Msg::ToggleDarkMode)
            on_add=move || send(Msg::ShowAddModal(true))
            on_scan=move || send(Msg::ShowScanner(true))
            on_settings=move || send(Msg::ShowSettings(true))
        />

        // Botanical background art — subtle fixed orchid accent
        <div class="overflow-hidden fixed inset-0 z-0 pointer-events-none">
            <div class="absolute -bottom-4 -right-8 text-primary botanical-breathe">
                <OrchidAccent class="w-64 h-auto sm:w-72" />
            </div>
        </div>

        <main class="relative z-10 py-6 px-4 mx-auto sm:px-6 max-w-[1200px]">
            <Suspense fallback=|| ()>
                {move || {
                    let readings = climate_readings.get();
                    view! { <ClimateDashboard readings=readings unit=temp_unit /> }
                }}
            </Suspense>

            <OrchidCollection
                orchids_resource=orchids_resource
                zones=zones_memo
                view_mode=view_mode
                on_set_view=move |mode| send(Msg::SetViewMode(mode))
                on_delete=on_delete
                on_select=move |o: Orchid| send(Msg::SelectOrchid(Some(o)))
                on_update=on_update
                on_add=move || send(Msg::ShowAddModal(true))
                on_scan=move || send(Msg::ShowScanner(true))
            />

            {move || show_add_modal.get().then(|| {
                let current_zones = zones_memo.get();
                view! {
                    <AddOrchidForm
                        zones=current_zones
                        on_add=on_add
                        on_close=move || send(Msg::ShowAddModal(false))
                        prefill_data=prefill_data
                    />
                }.into_any()
            })}

            {move || selected_orchid.get().map(|orchid| {
                let current_zones = zones_memo.get();
                view! {
                    <OrchidDetail
                        orchid=orchid
                        zones=current_zones
                        on_close=move || send(Msg::SelectOrchid(None))
                        on_update=on_update
                    />
                }.into_any()
            })}

            {move || show_settings.get().then(|| {
                let current_zones = zones_memo.get();
                view! {
                    <SettingsModal
                        zones=current_zones
                        on_close=move |temp_unit: String| send(Msg::SettingsClosed { temp_unit })
                        on_zones_changed=on_zones_changed
                    />
                }.into_any()
            })}

            {move || show_scanner.get().then(|| {
                let orchids = orchids_resource.get().and_then(|r| r.ok()).unwrap_or_default();
                let current_zones = zones_memo.get();
                let current_readings = climate_readings.get();
                view! {
                    <ScannerModal
                        on_close=move || send(Msg::ShowScanner(false))
                        on_add_to_collection=move |result| send(Msg::HandleScanResult(result))
                        existing_orchids=orchids
                        climate_readings=current_readings
                        zones=current_zones
                    />
                }.into_any()
            })}
        </main>
    }
}
