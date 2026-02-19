use leptos::prelude::*;
use crate::components::add_orchid_form::AddOrchidForm;
use crate::components::app_header::AppHeader;
use crate::components::botanical_art::OrchidAccent;
use crate::components::climate_dashboard::ClimateDashboard;
use crate::components::notification_setup::NotificationSetup;
use crate::components::orchid_collection::OrchidCollection;
use crate::components::orchid_detail::OrchidDetail;
use crate::components::scanner::ScannerModal;
use crate::components::settings::SettingsModal;
use crate::orchid::Alert;
use crate::model::{Model, Msg};
use crate::orchid::Orchid;
use crate::server_fns::auth::get_current_user;
use crate::server_fns::orchids::{get_orchids, create_orchid, update_orchid, delete_orchid, mark_watered};
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

    // Active alerts
    let alerts_resource = Resource::new(
        move || zones_version.get(),
        |_| crate::server_fns::alerts::get_active_alerts(),
    );

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
                orchid.native_region,
                orchid.native_latitude,
                orchid.native_longitude,
                orchid.temp_min,
                orchid.temp_max,
                orchid.humidity_min,
                orchid.humidity_max,
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

    let on_water = move |id: String| {
        leptos::task::spawn_local(async move {
            let _ = mark_watered(id).await;
            orchids_resource.refetch();
        });
    };

    let on_zones_changed = move || {
        set_zones_version.update(|v| *v += 1);
    };

    view! {
        <Suspense fallback=move || view! { <p class="p-8 text-center text-stone-500">"Loading..."</p> }>
            {move || {
                // Read ALL resources here so the outer Suspense tracks them.
                // The Suspense won't resolve until every resource has data,
                // ensuring inner Memos/Suspenses see the same values during
                // hydration as they did during SSR (preventing DOM mismatches).
                let _ = orchids_resource.get();
                let _ = climate_resource.get();
                let _ = alerts_resource.get();
                let _ = migration_resource.get();
                let _ = zones_resource.get();

                user.get().map(|result| match result {
                    Ok(Some(_user_info)) => {
                        // Check if user needs onboarding (no zones)
                        let zones = zones_memo.get();
                        if zones.is_empty()
                            && migration_resource.get().is_some()
                            && zones_resource.get().is_some()
                        {
                            #[cfg(feature = "ssr")]
                            leptos_axum::redirect("/onboarding");
                            #[cfg(feature = "hydrate")]
                            {
                                if let Some(window) = web_sys::window() {
                                    let _ = window.location().set_href("/onboarding");
                                }
                            }
                            return view! { <div></div> }.into_any();
                        }

                        // Authenticated user with zones — render full page
                        view! {
                            <AppHeader
                                dark_mode=dark_mode
                                on_toggle_dark=move || send(Msg::ToggleDarkMode)
                                on_add=move || send(Msg::ShowAddModal(true))
                                on_scan=move || send(Msg::ShowScanner(true))
                                on_settings=move || send(Msg::ShowSettings(true))
                            />

                            // Botanical background art + subtle green glow
                            <div class="overflow-hidden fixed inset-0 z-0 pointer-events-none">
                                <div class="absolute top-0 right-0 left-0 h-64 bg-gradient-to-b to-transparent from-primary/[0.04] dark:from-primary-light/[0.06]"></div>
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

                                <NotificationSetup />

                                <Suspense fallback=|| ()>
                                    {move || {
                                        alerts_resource.get().map(|result| {
                                            let alerts = result.unwrap_or_default();
                                            if alerts.is_empty() {
                                                view! { <div></div> }.into_any()
                                            } else {
                                                view! { <AlertBanner alerts=alerts on_dismiss=move |id: String| {
                                                    leptos::task::spawn_local(async move {
                                                        let _ = crate::server_fns::alerts::acknowledge_alert(id).await;
                                                        alerts_resource.refetch();
                                                    });
                                                } /> }.into_any()
                                            }
                                        })
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
                                    on_water=on_water
                                    on_add=move || send(Msg::ShowAddModal(true))
                                    on_scan=move || send(Msg::ShowScanner(true))
                                />
                            </main>

                            // Modals rendered outside <main> to avoid stacking context constraints
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
                                let current_readings = climate_readings.get();
                                view! {
                                    <OrchidDetail
                                        orchid=orchid
                                        zones=current_zones
                                        climate_readings=current_readings
                                        on_close=move || send(Msg::SelectOrchid(None))
                                        on_update=on_update
                                    />
                                }.into_any()
                            })}

                            {move || show_settings.get().then(|| {
                                let current_zones = zones_memo.get();
                                let current_temp_unit = temp_unit.get();
                                view! {
                                    <SettingsModal
                                        zones=current_zones
                                        initial_temp_unit=current_temp_unit
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
                        }.into_any()
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
    }
}

/// Alert banner showing active condition/watering alerts
#[component]
fn AlertBanner(
    alerts: Vec<Alert>,
    on_dismiss: impl Fn(String) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-2 mb-4">
            {alerts.into_iter().map(|alert| {
                let id = alert.id.clone();
                let (bg, text, border) = match alert.severity.as_str() {
                    "critical" => ("bg-red-50 dark:bg-red-900/20", "text-red-700 dark:text-red-300", "border-red-200 dark:border-red-800"),
                    "warning" => ("bg-amber-50 dark:bg-amber-900/20", "text-amber-700 dark:text-amber-300", "border-amber-200 dark:border-amber-800"),
                    _ => ("bg-sky-50 dark:bg-sky-900/20", "text-sky-700 dark:text-sky-300", "border-sky-200 dark:border-sky-800"),
                };
                let class = format!("flex gap-3 justify-between items-center p-3 text-sm rounded-xl border {} {} {}", bg, text, border);
                view! {
                    <div class=class>
                        <span>{alert.message}</span>
                        <button
                            class="py-1 px-2 text-xs rounded-lg border-none opacity-60 transition-opacity cursor-pointer hover:opacity-100 bg-black/5"
                            on:click=move |_| on_dismiss(id.clone())
                        >"Dismiss"</button>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }.into_any()
}
