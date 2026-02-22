use leptos::prelude::*;
use crate::components::add_orchid_form::AddOrchidForm;
use crate::components::app_header::AppHeader;
use crate::components::botanical_art::OrchidAccent;
use crate::components::climate_strip::ClimateStrip;
use crate::components::zone_wizard::ZoneConditionWizard;
use crate::components::notification_setup::NotificationSetup;
use crate::components::orchid_collection::OrchidCollection;
use crate::components::orchid_detail::OrchidDetail;
use crate::components::seasonal_calendar::SeasonalCalendar;
use crate::components::scanner::ScannerModal;
use crate::components::settings::SettingsModal;
use crate::orchid::Alert;
use crate::model::{HomeTab, Model, Msg};
use crate::orchid::Orchid;
use crate::server_fns::auth::get_current_user;
use crate::server_fns::orchids::{get_orchids, create_orchid, update_orchid, delete_orchid, mark_watered};
use crate::server_fns::preferences::{get_temp_unit, get_hemisphere, get_collection_public};
use crate::server_fns::devices::get_devices;
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

    // Load hardware devices
    let devices_resource = Resource::new(
        move || zones_version.get(), // reload when zones change (devices may be added)
        |_| get_devices(),
    );
    let devices_memo = Memo::new(move |_| {
        devices_resource.get()
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
    let wizard_zone = Memo::new(move |_| model.get().wizard_zone.clone());
    let home_tab = Memo::new(move |_| model.get().home_tab);

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

    // Load saved temp unit preference from server
    let temp_unit_resource = Resource::new(|| (), |_| get_temp_unit());
    let hemisphere_resource = Resource::new(|| (), |_| get_hemisphere());
    let collection_public_resource = Resource::new(|| (), |_| get_collection_public());

    // Initialize model temp_unit from server preference when it loads
    Effect::new(move |_| {
        if let Some(Ok(unit)) = temp_unit_resource.get() {
            set_model.update(|m| {
                if m.temp_unit != unit {
                    m.temp_unit = unit;
                }
            });
        }
    });

    // Initialize model hemisphere from server preference when it loads
    Effect::new(move |_| {
        if let Some(Ok(hemi)) = hemisphere_resource.get() {
            set_model.update(|m| {
                if m.hemisphere != hemi {
                    m.hemisphere = hemi;
                }
            });
        }
    });

    let hemisphere = Memo::new(move |_| model.get().hemisphere.clone());

    // Error toast signal
    let (toast_msg, set_toast_msg) = signal::<Option<String>>(None);

    // Orchid operations via server functions (async I/O — not TEA state)
    let on_add = move |orchid: Orchid| {
        leptos::task::spawn_local(async move {
            match create_orchid(
                orchid.name,
                orchid.species,
                orchid.water_frequency_days,
                orchid.light_requirement.as_str().to_string(),
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
                orchid.fertilize_frequency_days,
                orchid.fertilizer_type,
                orchid.pot_medium,
                orchid.pot_size,
                orchid.rest_start_month,
                orchid.rest_end_month,
                orchid.bloom_start_month,
                orchid.bloom_end_month,
                orchid.rest_water_multiplier,
                orchid.rest_fertilizer_multiplier,
                orchid.active_water_multiplier,
                orchid.active_fertilizer_multiplier,
            ).await {
                Ok(_) => {},
                Err(e) => set_toast_msg.set(Some(format!("Failed to add plant: {}", e))),
            }
            orchids_resource.refetch();
        });
    };

    let on_update = move |orchid: Orchid| {
        leptos::task::spawn_local(async move {
            if let Err(e) = update_orchid(orchid.clone()).await {
                set_toast_msg.set(Some(format!("Failed to update plant: {}", e)));
            }
            orchids_resource.refetch();
        });
    };

    let on_delete = move |id: String| {
        #[cfg(feature = "hydrate")]
        {
            if let Some(window) = web_sys::window() {
                if !window.confirm_with_message("Are you sure you want to delete this plant?").unwrap_or(false) {
                    return;
                }
            }
        }
        leptos::task::spawn_local(async move {
            if let Err(e) = delete_orchid(id).await {
                set_toast_msg.set(Some(format!("Failed to delete plant: {}", e)));
            }
            orchids_resource.refetch();
        });
    };

    let on_water = move |id: String| {
        leptos::task::spawn_local(async move {
            if let Err(e) = mark_watered(id).await {
                set_toast_msg.set(Some(format!("Failed to mark watered: {}", e)));
            }
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
                let _ = temp_unit_resource.get();
                let _ = hemisphere_resource.get();
                let _ = collection_public_resource.get();

                user.get().map(|result| match result {
                    Ok(Some(ref _user_info)) => {
                        let current_username = _user_info.username.clone();
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
                                // Tab bar
                                <div class="flex mb-5 border-b border-stone-200 dark:border-stone-700">
                                    <button
                                        class=move || if home_tab.get() == HomeTab::MyPlants {
                                            "flex gap-2 items-center py-2.5 px-5 text-sm font-semibold border-b-2 cursor-pointer transition-colors text-primary border-primary dark:text-primary-light dark:border-primary-light"
                                        } else {
                                            "flex gap-2 items-center py-2.5 px-5 text-sm font-medium border-b-2 border-transparent cursor-pointer transition-colors text-stone-400 hover:text-stone-600 dark:text-stone-500 dark:hover:text-stone-300"
                                        }
                                        on:click=move |_| send(Msg::SetHomeTab(HomeTab::MyPlants))
                                    >
                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                                            <path d="M10.394 2.08a1 1 0 00-.788 0l-7 3a1 1 0 000 1.84L5.25 8.051a.999.999 0 01.356-.257l4-1.714a1 1 0 11.788 1.838L7.667 9.088l1.94.831a1 1 0 00.787 0l7-3a1 1 0 000-1.838l-7-3zM3.31 9.397L5 10.12v4.102a8.969 8.969 0 00-1.05-.174 1 1 0 01-.89-.89 11.115 11.115 0 01.25-3.762zM9.3 16.573A9.026 9.026 0 007 14.935v-3.957l1.818.78a3 3 0 002.364 0l5.508-2.361a11.026 11.026 0 01.25 3.762 1 1 0 01-.89.89 8.968 8.968 0 00-5.35 2.524 1 1 0 01-1.4 0zM6 18a1 1 0 001-1v-2.065a8.935 8.935 0 00-2-.712V17a1 1 0 001 1z" />
                                        </svg>
                                        "My Plants"
                                    </button>
                                    <button
                                        class=move || if home_tab.get() == HomeTab::Seasons {
                                            "flex gap-2 items-center py-2.5 px-5 text-sm font-semibold border-b-2 cursor-pointer transition-colors text-primary border-primary dark:text-primary-light dark:border-primary-light"
                                        } else {
                                            "flex gap-2 items-center py-2.5 px-5 text-sm font-medium border-b-2 border-transparent cursor-pointer transition-colors text-stone-400 hover:text-stone-600 dark:text-stone-500 dark:hover:text-stone-300"
                                        }
                                        on:click=move |_| send(Msg::SetHomeTab(HomeTab::Seasons))
                                    >
                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                                            <path fill-rule="evenodd" d="M6 2a1 1 0 00-1 1v1H4a2 2 0 00-2 2v10a2 2 0 002 2h12a2 2 0 002-2V6a2 2 0 00-2-2h-1V3a1 1 0 10-2 0v1H7V3a1 1 0 00-1-1zm0 5a1 1 0 000 2h8a1 1 0 100-2H6z" clip-rule="evenodd" />
                                        </svg>
                                        "Seasons"
                                    </button>
                                </div>

                                // Tab content
                                {move || {
                                    match home_tab.get() {
                                        HomeTab::MyPlants => view! {
                                            <div>
                                                <Suspense fallback=|| ()>
                                                    {move || {
                                                        let readings = climate_readings.get();
                                                        let current_zones = zones_memo.get();
                                                        let tu = temp_unit.get();
                                                        view! { <ClimateStrip
                                                            readings=readings
                                                            zones=current_zones
                                                            unit=temp_unit
                                                            on_show_wizard=move |z| send(Msg::ShowWizard(Some(z)))
                                                            on_zones_changed=on_zones_changed
                                                            temp_unit_str=tu
                                                        /> }
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
                                            </div>
                                        }.into_any(),
                                        HomeTab::Seasons => view! {
                                            <div>
                                                <Suspense fallback=|| ()>
                                                    {move || {
                                                        let orchids = orchids_resource.get()
                                                            .and_then(|r| r.ok())
                                                            .unwrap_or_default();
                                                        let hemi = hemisphere.get();
                                                        view! { <SeasonalCalendar orchids=orchids hemisphere=hemi /> }
                                                    }}
                                                </Suspense>
                                            </div>
                                        }.into_any(),
                                    }
                                }}
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
                                let current_hemi = hemisphere.get();
                                view! {
                                    <OrchidDetail
                                        orchid=orchid
                                        zones=current_zones
                                        climate_readings=current_readings
                                        hemisphere=current_hemi
                                        on_close=move || send(Msg::SelectOrchid(None))
                                        on_update=on_update
                                    />
                                }.into_any()
                            })}

                            {move || show_settings.get().then(|| {
                                let current_zones = zones_memo.get();
                                let current_devices = devices_memo.get();
                                let current_temp_unit = temp_unit.get();
                                let current_hemi = hemisphere.get();
                                let current_public = collection_public_resource.get()
                                    .and_then(|r| r.ok())
                                    .unwrap_or(false);
                                let uname = current_username.clone();
                                view! {
                                    <SettingsModal
                                        zones=current_zones
                                        devices=current_devices
                                        initial_temp_unit=current_temp_unit.clone()
                                        initial_hemisphere=current_hemi
                                        initial_collection_public=current_public
                                        username=uname
                                        on_close=move |new_unit: String| {
                                    send(Msg::SettingsClosed { temp_unit: new_unit });
                                }
                                        on_zones_changed=on_zones_changed
                                        on_show_wizard=move |z| send(Msg::ShowWizard(Some(z)))
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

                            {move || wizard_zone.get().map(|zone| {
                                let current_unit = temp_unit.get();
                                view! {
                                    <ZoneConditionWizard
                                        zone=zone
                                        temp_unit=current_unit
                                        on_close=move || send(Msg::ShowWizard(None))
                                        on_saved=move || {
                                            on_zones_changed();
                                            send(Msg::ShowWizard(None));
                                        }
                                    />
                                }.into_any()
                            })}

                            <ErrorToast msg=toast_msg set_msg=set_toast_msg />
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

/// Error toast — botanical-themed notification with organic spring animation,
/// glassmorphic backdrop, progress drain bar, and 5-second auto-dismiss.
#[component]
fn ErrorToast(
    msg: ReadSignal<Option<String>>,
    set_msg: WriteSignal<Option<String>>,
) -> impl IntoView {
    view! {
        {move || msg.get().map(|text| {
            // Auto-dismiss after 5 seconds (hydrate-only)
            #[cfg(feature = "hydrate")]
            {
                let dismiss = set_msg;
                leptos::task::spawn_local(async move {
                    gloo_timers::future::TimeoutFuture::new(5_000).await;
                    dismiss.set(None);
                });
            }

            view! {
                <div class="fixed right-3 left-3 bottom-4 z-50 sm:left-4 sm:right-auto sm:max-w-sm toast-enter">
                    <div class="overflow-hidden relative rounded-2xl border shadow-xl backdrop-blur-md bg-surface/90 border-danger/20 dark:bg-stone-900/90 dark:border-danger/30">
                        // Warm danger gradient along the left edge
                        <div class="absolute top-0 bottom-0 left-0 w-1 bg-gradient-to-b from-danger via-danger/70 to-danger/30"></div>

                        <div class="flex gap-3 items-start py-3.5 pr-3 pl-5">
                            // Pulsing warning icon
                            <span class="flex-shrink-0 mt-0.5 text-lg text-danger toast-icon-pulse" aria-hidden="true">
                                "\u{26A0}"
                            </span>

                            <div class="flex-1 min-w-0">
                                <p class="text-xs font-semibold tracking-wide uppercase text-danger/80 dark:text-danger/90">"Something went wrong"</p>
                                <p class="mt-0.5 text-sm leading-snug text-stone-700 dark:text-stone-300">{text}</p>
                            </div>

                            // Dismiss button — subtle, stone-toned
                            <button
                                class="flex-shrink-0 p-1.5 mt-0.5 rounded-lg border-none transition-colors cursor-pointer text-stone-400 dark:hover:text-stone-200 dark:hover:bg-stone-800 hover:text-stone-600 hover:bg-stone-100"
                                on:click=move |_| set_msg.set(None)
                                aria-label="Dismiss"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                                    <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                                </svg>
                            </button>
                        </div>

                        // Progress drain bar — visually counts down the auto-dismiss
                        <div class="h-0.5 bg-danger/10 dark:bg-danger/5">
                            <div class="h-full rounded-r-full toast-progress bg-danger/40"></div>
                        </div>
                    </div>
                </div>
            }
        })}
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
