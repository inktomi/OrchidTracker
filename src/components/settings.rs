use leptos::prelude::*;
use crate::orchid::{GrowingZone, HardwareDevice};
use super::{MODAL_OVERLAY, MODAL_CONTENT, MODAL_HEADER, BTN_PRIMARY, BTN_CLOSE, BTN_SECONDARY, BTN_DANGER};

const INPUT_SM: &str = "w-full px-3 py-2 text-sm bg-white/80 border border-stone-300/50 rounded-lg outline-none transition-all duration-200 placeholder:text-stone-400 focus:bg-white focus:border-primary/40 focus:ring-2 focus:ring-primary/10 dark:bg-stone-800/80 dark:border-stone-600/50 dark:placeholder:text-stone-500 dark:focus:bg-stone-800 dark:focus:border-primary-light/40 dark:focus:ring-primary-light/10";
const LABEL_SM: &str = "block mb-1 text-xs font-semibold tracking-wider uppercase text-stone-400 dark:text-stone-500";
const BTN_SM: &str = "py-1.5 px-3 text-xs font-semibold rounded-lg border-none cursor-pointer transition-colors";

#[component]
pub fn SettingsModal(
    zones: Vec<GrowingZone>,
    #[prop(default = vec![])]
    devices: Vec<HardwareDevice>,
    initial_temp_unit: String,
    initial_hemisphere: String,
    #[prop(optional)] initial_collection_public: bool,
    #[prop(optional)] username: String,
    on_close: impl Fn(String) + 'static + Copy + Send + Sync,
    on_zones_changed: impl Fn() + 'static + Copy + Send + Sync,
    on_show_wizard: impl Fn(GrowingZone) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let (temp_unit, set_temp_unit) = signal(initial_temp_unit);
    let (hemisphere, set_hemisphere) = signal(initial_hemisphere);
    let (collection_public, set_collection_public) = signal(initial_collection_public);
    let username_stored = StoredValue::new(username);
    let (local_devices, set_local_devices) = signal(devices);

    // Zone management state
    let (show_add_zone, set_show_add_zone) = signal(false);
    let (add_name, set_add_name) = signal(String::new());
    let (add_light, set_add_light) = signal("Medium".to_string());
    let (add_location, set_add_location) = signal("Indoor".to_string());
    let (add_temp, set_add_temp) = signal(String::new());
    let (add_humidity, set_add_humidity) = signal(String::new());
    let (add_desc, set_add_desc) = signal(String::new());

    let (is_zone_saving, set_is_zone_saving) = signal(false);
    let (local_zones, set_local_zones) = signal(zones);

    let reset_add_form = move || {
        set_add_name.set(String::new());
        set_add_light.set("Medium".to_string());
        set_add_location.set("Indoor".to_string());
        set_add_temp.set(String::new());
        set_add_humidity.set(String::new());
        set_add_desc.set(String::new());
        set_show_add_zone.set(false);
    };

    let add_zone = move |_| {
        let name = add_name.get();
        if name.is_empty() { return; }
        set_is_zone_saving.set(true);

        let light = add_light.get();
        let location = add_location.get();
        let temp = add_temp.get();
        let humidity = add_humidity.get();
        let desc = add_desc.get();
        let sort_order = local_zones.get().len() as i32;

        leptos::task::spawn_local(async move {
            match crate::server_fns::zones::create_zone(
                name, light, location, temp, humidity, desc, sort_order,
            ).await {
                Ok(zone) => {
                    set_local_zones.update(|z| z.push(zone));
                    on_zones_changed();
                }
                Err(e) => {
                    log::error!("Failed to create zone: {}", e);
                }
            }
            set_is_zone_saving.set(false);
            reset_add_form();
        });
    };

    let delete_zone = move |id: String| {
        set_is_zone_saving.set(true);
        let zone_id = id.clone();
        leptos::task::spawn_local(async move {
            match crate::server_fns::zones::delete_zone(zone_id.clone()).await {
                Ok(()) => {
                    set_local_zones.update(|z| z.retain(|zone| zone.id != zone_id));
                    on_zones_changed();
                }
                Err(e) => {
                    log::error!("Failed to delete zone: {}", e);
                }
            }
            set_is_zone_saving.set(false);
        });
    };

    view! {
        <div class=MODAL_OVERLAY>
            <div class=MODAL_CONTENT>
                <div class=MODAL_HEADER>
                    <h2 class="m-0">"Settings"</h2>
                    <button class=BTN_CLOSE on:click=move |_| on_close(temp_unit.get_untracked())>"Close"</button>
                </div>
                <div>
                    <div class="mb-4">
                        <label>"Hemisphere:"</label>
                        <select
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_hemisphere.set(val.clone());
                                leptos::task::spawn_local(async move {
                                    let _ = crate::server_fns::preferences::save_hemisphere(val).await;
                                });
                            }
                            prop:value=hemisphere
                        >
                            <option value="N">"Northern Hemisphere"</option>
                            <option value="S">"Southern Hemisphere"</option>
                        </select>
                    </div>
                    <div class="mb-4">
                        <label>"Temperature Unit:"</label>
                        <select
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_temp_unit.set(val.clone());
                                leptos::task::spawn_local(async move {
                                    let _ = crate::server_fns::preferences::save_temp_unit(val).await;
                                });
                            }
                            prop:value=temp_unit
                        >
                            <option value="C">"Celsius (C)"</option>
                            <option value="F">"Fahrenheit (F)"</option>
                        </select>
                    </div>

                    <hr class="my-6 border-stone-200 dark:border-stone-700" />

                    // Public Collection toggle
                    <div class="mb-6">
                        <h3 class="mb-4 text-sm font-semibold tracking-wider uppercase text-stone-500 dark:text-stone-400">"Public Collection"</h3>
                        <div class="flex flex-col gap-3">
                            <div class="flex justify-between items-center">
                                <div>
                                    <div class="text-sm font-medium text-stone-700 dark:text-stone-300">"Share your collection publicly"</div>
                                    <div class="text-xs text-stone-400">"Allow anyone with the link to view your plants (read-only)"</div>
                                </div>
                                <button
                                    class=move || if collection_public.get() {
                                        "relative w-11 h-6 bg-primary rounded-full transition-colors cursor-pointer border-none"
                                    } else {
                                        "relative w-11 h-6 bg-stone-300 dark:bg-stone-600 rounded-full transition-colors cursor-pointer border-none"
                                    }
                                    on:click=move |_| {
                                        let new_val = !collection_public.get();
                                        set_collection_public.set(new_val);
                                        leptos::task::spawn_local(async move {
                                            let _ = crate::server_fns::preferences::save_collection_public(new_val).await;
                                        });
                                    }
                                >
                                    <span class=move || if collection_public.get() {
                                        "absolute top-0.5 left-5.5 w-5 h-5 bg-white rounded-full transition-all shadow-sm"
                                    } else {
                                        "absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-all shadow-sm"
                                    }></span>
                                </button>
                            </div>
                            {move || collection_public.get().then(|| {
                                let uname = username_stored.get_value();
                                let url = format!("/u/{}", uname);
                                view! {
                                    <div class="p-3 text-sm rounded-lg bg-primary/5 dark:bg-primary-light/5">
                                        <div class="text-xs font-medium text-stone-500 dark:text-stone-400">"Shareable link:"</div>
                                        <code class="text-sm text-primary dark:text-primary-light">{url}</code>
                                    </div>
                                }
                            })}
                        </div>
                    </div>

                    <hr class="my-6 border-stone-200 dark:border-stone-700" />

                    // Hardware Devices section
                    <div class="mb-6">
                        <h3 class="mb-4 text-sm font-semibold tracking-wider uppercase text-stone-500 dark:text-stone-400">"Hardware Devices"</h3>
                        <crate::components::device_management::DeviceList
                            devices=local_devices
                            set_devices=set_local_devices
                        />
                    </div>

                    <hr class="my-6 border-stone-200 dark:border-stone-700" />

                    // Growing Zones section
                    <div class="mb-6">
                        <h3 class="mb-4 text-sm font-semibold tracking-wider uppercase text-stone-500 dark:text-stone-400">"Growing Zones"</h3>

                        <div class="flex flex-col gap-2 mb-4">
                            <For
                                each=move || local_zones.get()
                                key=|zone| zone.id.clone()
                                children=move |zone| {
                                    view! { <ZoneCard zone=zone on_delete=delete_zone on_zones_changed=on_zones_changed is_saving=is_zone_saving set_local_zones=set_local_zones on_show_wizard=on_show_wizard temp_unit=temp_unit devices=local_devices /> }
                                }
                            />
                        </div>

                        // Add zone form
                        {move || if show_add_zone.get() {
                            view! {
                                <div class="p-4 mb-4 rounded-xl border bg-secondary/30 border-stone-200/60 dark:border-stone-700">
                                    <div class="mb-3">
                                        <label class=LABEL_SM>"Name"</label>
                                        <input type="text" class=INPUT_SM
                                            placeholder="e.g. Kitchen Windowsill"
                                            prop:value=add_name
                                            on:input=move |ev| set_add_name.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="flex gap-3 mb-3">
                                        <div class="flex-1">
                                            <label class=LABEL_SM>"Light"</label>
                                            <select class=INPUT_SM
                                                prop:value=add_light
                                                on:change=move |ev| set_add_light.set(event_target_value(&ev))
                                            >
                                                <option value="Low">"Low"</option>
                                                <option value="Medium">"Medium"</option>
                                                <option value="High">"High"</option>
                                            </select>
                                        </div>
                                        <div class="flex-1">
                                            <label class=LABEL_SM>"Location"</label>
                                            <select class=INPUT_SM
                                                prop:value=add_location
                                                on:change=move |ev| set_add_location.set(event_target_value(&ev))
                                            >
                                                <option value="Indoor">"Indoor"</option>
                                                <option value="Outdoor">"Outdoor"</option>
                                            </select>
                                        </div>
                                    </div>
                                    <div class="flex gap-3 mb-3">
                                        <div class="flex-1">
                                            <label class=LABEL_SM>"Temp Range"</label>
                                            <input type="text" class=INPUT_SM
                                                placeholder="e.g. 18-28C"
                                                prop:value=add_temp
                                                on:input=move |ev| set_add_temp.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div class="flex-1">
                                            <label class=LABEL_SM>"Humidity"</label>
                                            <input type="text" class=INPUT_SM
                                                placeholder="e.g. 50-70%"
                                                prop:value=add_humidity
                                                on:input=move |ev| set_add_humidity.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>
                                    <div class="mb-3">
                                        <label class=LABEL_SM>"Description"</label>
                                        <input type="text" class=INPUT_SM
                                            placeholder="Optional notes"
                                            prop:value=add_desc
                                            on:input=move |ev| set_add_desc.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="flex gap-2">
                                        <button class=BTN_PRIMARY
                                            disabled=move || is_zone_saving.get()
                                            on:click=add_zone
                                        >"Add"</button>
                                        <button class=BTN_SECONDARY
                                            on:click=move |_| reset_add_form()
                                        >"Cancel"</button>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <button
                                    class="flex gap-2 justify-center items-center py-2 mb-4 w-full text-sm font-medium rounded-xl border border-dashed transition-colors cursor-pointer text-stone-400 border-stone-300 dark:border-stone-600 hover:text-primary hover:border-primary/40"
                                    on:click=move |_| set_show_add_zone.set(true)
                                >
                                    "+  Add Zone"
                                </button>
                            }.into_any()
                        }}
                    </div>

                    <hr class="my-6 border-stone-200 dark:border-stone-700" />

                    // Notifications section
                    <div class="mb-6">
                        <h3 class="mb-4 text-sm font-semibold tracking-wider uppercase text-stone-500 dark:text-stone-400">"Notifications"</h3>
                        <NotificationSettings />
                    </div>

                </div>
            </div>
        </div>
    }
}

/// Individual zone card with data source configuration
#[component]
fn ZoneCard(
    zone: GrowingZone,
    on_delete: impl Fn(String) + 'static + Copy + Send + Sync,
    on_zones_changed: impl Fn() + 'static + Copy + Send + Sync,
    is_saving: ReadSignal<bool>,
    set_local_zones: WriteSignal<Vec<GrowingZone>>,
    on_show_wizard: impl Fn(GrowingZone) + 'static + Copy + Send + Sync,
    temp_unit: ReadSignal<String>,
    devices: ReadSignal<Vec<HardwareDevice>>,
) -> impl IntoView {
    let zone_id_for_delete = zone.id.clone();
    let zone_id_for_config = zone.id.clone();
    let zone_for_wizard = zone.clone();
    let zone_for_manual = zone.clone();

    let light_class = match zone.light_level {
        crate::orchid::LightRequirement::High => "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-300",
        crate::orchid::LightRequirement::Low => "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300",
        _ => "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300",
    };
    let loc_class = match zone.location_type {
        crate::orchid::LocationType::Outdoor => "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-sky-100 text-sky-700 dark:bg-sky-900/30 dark:text-sky-300",
        _ => "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-stone-100 text-stone-600 dark:bg-stone-800 dark:text-stone-400",
    };

    let has_source = zone.data_source_type.is_some() || zone.hardware_device_id.is_some();
    let source_dot_class = if has_source {
        "inline-block w-2 h-2 rounded-full bg-emerald-500"
    } else {
        ""
    };

    let (show_config, set_show_config) = signal(false);
    let (show_manual, set_show_manual) = signal(false);

    view! {
        <div class="rounded-xl border bg-secondary/30 border-stone-200/60 dark:border-stone-700">
            <div class="flex justify-between items-center p-3">
                <div class="flex flex-col gap-1">
                    <div class="flex gap-2 items-center">
                        {(!source_dot_class.is_empty()).then(|| view! { <span class=source_dot_class></span> })}
                        <span class="text-sm font-medium text-stone-700 dark:text-stone-300">{zone.name.clone()}</span>
                    </div>
                    <div class="flex gap-2">
                        <span class=light_class>{zone.light_level.to_string()}</span>
                        <span class=loc_class>{zone.location_type.to_string()}</span>
                    </div>
                </div>
                <div class="flex flex-wrap gap-1.5">
                    {(!has_source).then(|| {
                        let z = zone_for_wizard.clone();
                        view! {
                            <button
                                class=format!("{} text-amber-600 bg-amber-50 hover:bg-amber-100 dark:text-amber-400 dark:bg-amber-900/20 dark:hover:bg-amber-900/40", BTN_SM)
                                on:click=move |_| on_show_wizard(z.clone())
                            >"Estimate"</button>
                        }
                    })}
                    <button
                        class=format!("{} text-sky-600 bg-sky-50 hover:bg-sky-100 dark:text-sky-400 dark:bg-sky-900/20 dark:hover:bg-sky-900/40", BTN_SM)
                        on:click=move |_| set_show_manual.update(|v| *v = !*v)
                    >{move || if show_manual.get() { "Cancel" } else { "Log" }}</button>
                    <button
                        class=format!("{} text-stone-500 bg-stone-100 hover:bg-stone-200 dark:text-stone-400 dark:bg-stone-800 dark:hover:bg-stone-700", BTN_SM)
                        on:click=move |_| set_show_config.update(|v| *v = !*v)
                    >
                        {move || if show_config.get() { "Hide" } else { "Configure" }}
                    </button>
                    <button
                        class=BTN_DANGER
                        disabled=move || is_saving.get()
                        on:click=move |_| on_delete(zone_id_for_delete.clone())
                    >"Delete"</button>
                </div>
            </div>

            {move || show_manual.get().then(|| {
                let z = zone_for_manual.clone();
                let unit = temp_unit.get();
                view! {
                    <div class="px-3 pb-3">
                        <crate::components::manual_reading::ManualReadingForm
                            zone=z
                            temp_unit=unit
                            on_saved=move || {
                                on_zones_changed();
                                set_show_manual.set(false);
                            }
                            on_cancel=move || set_show_manual.set(false)
                        />
                    </div>
                }
            })}

            {move || show_config.get().then(|| {
                view! {
                    <DataSourceConfig
                        zone_id=zone_id_for_config.clone()
                        current_type=zone.data_source_type.clone()
                        current_config=zone.data_source_config.clone()
                        current_hardware_device_id=zone.hardware_device_id.clone()
                        current_hardware_port=zone.hardware_port
                        on_saved=move || {
                            on_zones_changed();
                        }
                        set_local_zones=set_local_zones
                        devices=devices
                    />
                }
            })}
        </div>
    }
}

/// Data source configuration form for a single zone.
/// Supports three modes:
/// - Device-linked: tempest/ac_infinity via shared hardware_device (picker shown)
/// - Legacy direct: tempest/ac_infinity with zone-level credentials (when no devices exist)
/// - Weather API: always zone-level lat/lon config
#[component]
fn DataSourceConfig(
    zone_id: String,
    current_type: Option<String>,
    current_config: String,
    #[prop(default = None)]
    current_hardware_device_id: Option<String>,
    #[prop(default = None)]
    current_hardware_port: Option<i32>,
    on_saved: impl Fn() + 'static + Copy + Send + Sync,
    set_local_zones: WriteSignal<Vec<GrowingZone>>,
    devices: ReadSignal<Vec<HardwareDevice>>,
) -> impl IntoView {
    // Determine initial provider from hardware device or legacy data_source_type
    let initial_provider = if current_hardware_device_id.is_some() {
        "device_linked".to_string()
    } else {
        current_type.clone().unwrap_or_default()
    };

    let (provider, set_provider) = signal(initial_provider);

    // Device picker state
    let (selected_device_id, set_selected_device_id) = signal(
        current_hardware_device_id.clone().unwrap_or_default()
    );
    let (selected_port, set_selected_port) = signal(
        current_hardware_port.map(|p| p.to_string()).unwrap_or_else(|| "1".to_string())
    );

    // Parse existing config to initialize ALL legacy fields
    let parsed = serde_json::from_str::<serde_json::Value>(&current_config).ok();

    let get_str = |key: &str| -> String {
        parsed.as_ref()
            .and_then(|j| j.get(key))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    };

    // Tempest legacy fields
    let (tempest_station, set_tempest_station) = signal(get_str("station_id"));
    let (tempest_token, set_tempest_token) = signal(get_str("token"));

    // AC Infinity legacy fields
    let (aci_email, set_aci_email) = signal(get_str("email"));
    let (aci_password, set_aci_password) = signal(get_str("password"));
    let (aci_device, set_aci_device) = signal(get_str("device_id"));
    let init_port = parsed.as_ref()
        .and_then(|j| j.get("port"))
        .and_then(|v| v.as_u64())
        .map(|n| n.to_string())
        .unwrap_or_else(|| "1".to_string());
    let (aci_port, set_aci_port) = signal(init_port);

    // Weather API fields
    let get_f64 = |key: &str| -> String {
        parsed.as_ref()
            .and_then(|j| j.get(key))
            .and_then(|v| v.as_f64())
            .map(|n| format!("{}", n))
            .unwrap_or_default()
    };
    let (wa_lat, set_wa_lat) = signal(get_f64("latitude"));
    let (wa_lon, set_wa_lon) = signal(get_f64("longitude"));

    let (test_result, set_test_result) = signal::<Option<Result<String, String>>>(None);
    let (is_testing, set_is_testing) = signal(false);
    let (is_saving_ds, set_is_saving_ds) = signal(false);

    let had_source = current_type.is_some() || current_hardware_device_id.is_some();

    // Build legacy config JSON for direct zone-level saves
    let build_legacy_config_json = move || -> String {
        match provider.get().as_str() {
            "tempest" => serde_json::json!({
                "station_id": tempest_station.get(),
                "token": tempest_token.get(),
            }).to_string(),
            "ac_infinity" => serde_json::json!({
                "email": aci_email.get(),
                "password": aci_password.get(),
                "device_id": aci_device.get(),
                "port": aci_port.get().parse::<u32>().unwrap_or(1),
            }).to_string(),
            "weather_api" => serde_json::json!({
                "latitude": wa_lat.get().parse::<f64>().unwrap_or(0.0),
                "longitude": wa_lon.get().parse::<f64>().unwrap_or(0.0),
            }).to_string(),
            _ => String::new(),
        }
    };

    // Whether the current provider type has shared hardware devices available.
    let has_devices_for_provider = move || -> bool {
        let prov = provider.get();
        let devs = devices.get();
        match prov.as_str() {
            "tempest" => devs.iter().any(|d| d.device_type == "tempest"),
            "ac_infinity" => devs.iter().any(|d| d.device_type == "ac_infinity"),
            _ => false,
        }
    };

    let test_connection = move |_| {
        let prov = provider.get();
        if prov.is_empty() { return; }
        set_is_testing.set(true);
        set_test_result.set(None);

        // If using device picker and a device is selected, test via device
        if has_devices_for_provider() && !selected_device_id.get().is_empty() {
            let dev_id = selected_device_id.get();
            let devs = devices.get();
            if let Some(device) = devs.iter().find(|d| d.id == dev_id) {
                let dt = device.device_type.clone();
                let cfg = device.config.clone();
                leptos::task::spawn_local(async move {
                    match crate::server_fns::devices::test_device(dt, cfg).await {
                        Ok(msg) => set_test_result.set(Some(Ok(msg))),
                        Err(e) => set_test_result.set(Some(Err(e.to_string()))),
                    }
                    set_is_testing.set(false);
                });
            } else {
                set_test_result.set(Some(Err("No device selected".into())));
                set_is_testing.set(false);
            }
        } else {
            // Legacy direct config test
            let config = build_legacy_config_json();
            leptos::task::spawn_local(async move {
                match crate::server_fns::climate::test_data_source(prov, config).await {
                    Ok(msg) => set_test_result.set(Some(Ok(msg))),
                    Err(e) => set_test_result.set(Some(Err(e.to_string()))),
                }
                set_is_testing.set(false);
            });
        }
    };

    let zone_id_save = StoredValue::new(zone_id.clone());
    let do_save = move || {
        let prov = provider.get();
        let zid = zone_id_save.get_value();
        set_is_saving_ds.set(true);

        if prov.is_empty() {
            // Remove: unlink device + clear legacy config
            leptos::task::spawn_local(async move {
                let _ = crate::server_fns::devices::unlink_zone_from_device(zid.clone()).await;
                let _ = crate::server_fns::climate::configure_zone_data_source(
                    zid.clone(), None, String::new()
                ).await;
                set_local_zones.update(|zones| {
                    if let Some(z) = zones.iter_mut().find(|z| z.id == zid) {
                        z.data_source_type = None;
                        z.data_source_config = String::new();
                        z.hardware_device_id = None;
                        z.hardware_port = None;
                    }
                });
                set_test_result.set(Some(Ok("Data source removed".into())));
                on_saved();
                set_is_saving_ds.set(false);
            });
        } else if has_devices_for_provider() && !selected_device_id.get().is_empty() {
            // Device-linked save: link zone to shared device
            let dev_id = selected_device_id.get();
            let port = if prov == "ac_infinity" {
                Some(selected_port.get().parse::<i32>().unwrap_or(1))
            } else {
                None
            };

            let dev_id_for_update = dev_id.clone();
            leptos::task::spawn_local(async move {
                match crate::server_fns::devices::link_zone_to_device(
                    zid.clone(), dev_id.clone(), port,
                ).await {
                    Ok(()) => {
                        set_local_zones.update(|zones| {
                            if let Some(z) = zones.iter_mut().find(|z| z.id == zid) {
                                z.hardware_device_id = Some(dev_id_for_update);
                                z.hardware_port = port;
                                z.data_source_type = None;
                                z.data_source_config = String::new();
                            }
                        });
                        set_test_result.set(Some(Ok("Linked to device!".into())));
                        on_saved();
                    }
                    Err(e) => {
                        set_test_result.set(Some(Err(format!("Link failed: {}", e))));
                    }
                }
                set_is_saving_ds.set(false);
            });
        } else {
            // Legacy zone-level config save (weather_api, or tempest/ac_infinity without devices)
            let config = build_legacy_config_json();
            let provider_opt = Some(prov.clone());

            leptos::task::spawn_local(async move {
                // Unlink any existing device first
                let _ = crate::server_fns::devices::unlink_zone_from_device(zid.clone()).await;

                match crate::server_fns::climate::configure_zone_data_source(
                    zid.clone(), provider_opt.clone(), config,
                ).await {
                    Ok(()) => {
                        set_local_zones.update(|zones| {
                            if let Some(z) = zones.iter_mut().find(|z| z.id == zid) {
                                z.data_source_type = provider_opt;
                                z.data_source_config = String::new();
                                z.hardware_device_id = None;
                                z.hardware_port = None;
                            }
                        });
                        set_test_result.set(Some(Ok("Saved successfully!".into())));
                        on_saved();
                    }
                    Err(e) => {
                        set_test_result.set(Some(Err(format!("Save failed: {}", e))));
                    }
                }
                set_is_saving_ds.set(false);
            });
        }
    };

    view! {
        <div class="p-3 pt-0 border-t border-stone-200/40 dark:border-stone-700/40">
            <div class="mt-3 mb-3">
                <label class=LABEL_SM>"Data Source"</label>
                <select class=INPUT_SM
                    prop:value=provider
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        set_provider.set(val.clone());
                        set_test_result.set(None);
                        set_selected_device_id.set(String::new());
                    }
                >
                    <option value="">"None"</option>
                    <option value="tempest">"Tempest Weather Station"</option>
                    <option value="ac_infinity">"AC Infinity Controller"</option>
                    <option value="weather_api">"Weather API (Outdoor)"</option>
                </select>
            </div>

            {move || {
                let prov = provider.get();
                match prov.as_str() {
                    "tempest" => {
                        let filtered: Vec<HardwareDevice> = devices.get().into_iter()
                            .filter(|d| d.device_type == "tempest")
                            .collect();
                        if filtered.is_empty() {
                            // No shared devices — show legacy credential fields
                            view! {
                                <div class="p-3 mb-3 rounded-lg bg-sky-50/50 dark:bg-sky-900/10">
                                    <div class="mb-3">
                                        <label class=LABEL_SM>"Station ID"</label>
                                        <input type="text" class=INPUT_SM
                                            placeholder="e.g. 12345"
                                            prop:value=tempest_station
                                            on:input=move |ev| set_tempest_station.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label class=LABEL_SM>"API Token"</label>
                                        <input type="password" class=INPUT_SM
                                            placeholder="Your WeatherFlow API token"
                                            prop:value=tempest_token
                                            on:input=move |ev| set_tempest_token.set(event_target_value(&ev))
                                        />
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            // Shared devices available — show picker
                            view! {
                                <div class="p-3 mb-3 rounded-lg bg-sky-50/50 dark:bg-sky-900/10">
                                    <label class=LABEL_SM>"Device"</label>
                                    <select class=INPUT_SM
                                        prop:value=selected_device_id
                                        on:change=move |ev| set_selected_device_id.set(event_target_value(&ev))
                                    >
                                        <option value="">"Select device..."</option>
                                        {filtered.into_iter().map(|d| {
                                            let id = d.id.clone();
                                            view! { <option value=id>{d.name}</option> }
                                        }).collect::<Vec<_>>()}
                                    </select>
                                </div>
                            }.into_any()
                        }
                    }
                    "ac_infinity" => {
                        let filtered: Vec<HardwareDevice> = devices.get().into_iter()
                            .filter(|d| d.device_type == "ac_infinity")
                            .collect();
                        if filtered.is_empty() {
                            // No shared devices — show legacy credential fields
                            view! {
                                <div class="p-3 mb-3 rounded-lg bg-violet-50/50 dark:bg-violet-900/10">
                                    <div class="flex gap-3 mb-3">
                                        <div class="flex-1">
                                            <label class=LABEL_SM>"Email"</label>
                                            <input type="email" class=INPUT_SM
                                                placeholder="AC Infinity account email"
                                                prop:value=aci_email
                                                on:input=move |ev| set_aci_email.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div class="flex-1">
                                            <label class=LABEL_SM>"Password"</label>
                                            <input type="password" class=INPUT_SM
                                                placeholder="Account password"
                                                prop:value=aci_password
                                                on:input=move |ev| set_aci_password.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>
                                    <div class="flex gap-3">
                                        <div class="flex-1">
                                            <label class=LABEL_SM>"Device ID"</label>
                                            <input type="text" class=INPUT_SM
                                                placeholder="e.g. ABC123DEF"
                                                prop:value=aci_device
                                                on:input=move |ev| set_aci_device.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div class="w-20">
                                            <label class=LABEL_SM>"Port"</label>
                                            <input type="number" class=INPUT_SM
                                                min="1" max="10"
                                                prop:value=aci_port
                                                on:input=move |ev| set_aci_port.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            // Shared devices available — show picker
                            view! {
                                <div class="p-3 mb-3 rounded-lg bg-violet-50/50 dark:bg-violet-900/10">
                                    <div class="mb-3">
                                        <label class=LABEL_SM>"Device"</label>
                                        <select class=INPUT_SM
                                            prop:value=selected_device_id
                                            on:change=move |ev| set_selected_device_id.set(event_target_value(&ev))
                                        >
                                            <option value="">"Select device..."</option>
                                            {filtered.into_iter().map(|d| {
                                                let id = d.id.clone();
                                                view! { <option value=id>{d.name}</option> }
                                            }).collect::<Vec<_>>()}
                                        </select>
                                    </div>
                                    <div class="w-24">
                                        <label class=LABEL_SM>"Port"</label>
                                        <input type="number" class=INPUT_SM
                                            min="1" max="10"
                                            prop:value=selected_port
                                            on:input=move |ev| set_selected_port.set(event_target_value(&ev))
                                        />
                                    </div>
                                </div>
                            }.into_any()
                        }
                    }
                    "weather_api" => view! {
                        <div class="p-3 mb-3 rounded-lg bg-emerald-50/50 dark:bg-emerald-900/10">
                            <div class="flex gap-3">
                                <div class="flex-1">
                                    <label class=LABEL_SM>"Latitude"</label>
                                    <input type="number" class=INPUT_SM step="0.0001"
                                        placeholder="e.g. 37.7749"
                                        prop:value=wa_lat
                                        on:input=move |ev| set_wa_lat.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="flex-1">
                                    <label class=LABEL_SM>"Longitude"</label>
                                    <input type="number" class=INPUT_SM step="0.0001"
                                        placeholder="e.g. -122.4194"
                                        prop:value=wa_lon
                                        on:input=move |ev| set_wa_lon.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                    "device_linked" => {
                        // Currently linked to a device — show which one
                        let dev_id = selected_device_id.get();
                        let devs = devices.get();
                        let device_name = devs.iter()
                            .find(|d| d.id == dev_id)
                            .map(|d| format!("{} ({})", d.name, d.device_type))
                            .unwrap_or_else(|| "Unknown device".to_string());
                        view! {
                            <div class="p-3 mb-3 rounded-lg bg-emerald-50/50 dark:bg-emerald-900/10">
                                <p class="text-sm text-stone-600 dark:text-stone-400">
                                    "Linked to: " <strong>{device_name}</strong>
                                    {current_hardware_port.map(|p| format!(" (Port {})", p)).unwrap_or_default()}
                                </p>
                                <p class="mt-1 text-xs text-stone-400">"Change the data source type above to reconfigure."</p>
                            </div>
                        }.into_any()
                    }
                    _ => view! {
                        <p class="mb-3 text-xs text-stone-400 dark:text-stone-500">"No data source configured for this zone."</p>
                    }.into_any(),
                }
            }}

            // Test result display
            {move || test_result.get().map(|result| {
                match result {
                    Ok(msg) => view! {
                        <div class="p-2 mb-3 text-xs text-emerald-700 bg-emerald-50 rounded-lg dark:text-emerald-300 dark:bg-emerald-900/20">{msg}</div>
                    }.into_any(),
                    Err(msg) => view! {
                        <div class="p-2 mb-3 text-xs text-red-700 bg-red-50 rounded-lg dark:text-red-300 dark:bg-red-900/20">{msg}</div>
                    }.into_any(),
                }
            })}

            // Action buttons
            {move || {
                let prov = provider.get();
                if prov.is_empty() && had_source {
                    view! {
                        <div class="flex gap-2">
                            <button
                                class=format!("{} text-white bg-primary hover:bg-primary-dark", BTN_SM)
                                disabled=move || is_saving_ds.get()
                                on:click=move |_| do_save()
                            >"Remove Data Source"</button>
                        </div>
                    }.into_any()
                } else if !prov.is_empty() && prov != "device_linked" {
                    view! {
                        <div class="flex gap-2">
                            <button
                                class=format!("{} text-stone-600 bg-stone-100 hover:bg-stone-200 dark:text-stone-300 dark:bg-stone-700 dark:hover:bg-stone-600", BTN_SM)
                                disabled=move || is_testing.get()
                                on:click=test_connection
                            >
                                {move || if is_testing.get() { "Testing..." } else { "Test Connection" }}
                            </button>
                            <button
                                class=format!("{} text-white bg-primary hover:bg-primary-dark", BTN_SM)
                                disabled=move || is_saving_ds.get()
                                on:click=move |_| do_save()
                            >
                                {move || if is_saving_ds.get() { "Saving..." } else { "Save" }}
                            </button>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </div>
    }
}

/// Notification settings section within the settings modal
#[component]
fn NotificationSettings() -> impl IntoView {
    let (permission_status, set_permission_status) = signal("Checking...".to_string());
    let (is_enabled, set_is_enabled) = signal(false);
    let (is_testing, set_is_testing) = signal(false);

    // Check browser permission + server subscription state
    #[cfg(feature = "hydrate")]
    {
        Effect::new(move |_| {
            let perm = web_sys::Notification::permission();
            match perm {
                web_sys::NotificationPermission::Denied => {
                    set_permission_status.set("Denied (change in browser settings)".into());
                    set_is_enabled.set(false);
                }
                web_sys::NotificationPermission::Default => {
                    set_permission_status.set("Not yet requested".into());
                    set_is_enabled.set(false);
                }
                web_sys::NotificationPermission::Granted | _ => {
                    // Permission granted — check server for active subscription
                    leptos::task::spawn_local(async move {
                        match crate::server_fns::alerts::has_push_subscription().await {
                            Ok(true) => {
                                set_permission_status.set("Enabled".into());
                                set_is_enabled.set(true);
                            }
                            _ => {
                                set_permission_status.set("Disabled".into());
                                set_is_enabled.set(false);
                            }
                        }
                    });
                }
            }
        });
    }

    let toggle_notifications = move |_| {
        if is_enabled.get() {
            // Disable: unsubscribe browser PushManager + server record
            leptos::task::spawn_local(async move {
                #[cfg(feature = "hydrate")]
                { unsubscribe_browser_push().await; }
                let _ = crate::server_fns::alerts::unsubscribe_push().await;
                set_is_enabled.set(false);
                set_permission_status.set("Disabled".into());
            });
        } else {
            // Enable: request permission + subscribe
            #[cfg(feature = "hydrate")]
            {
                leptos::task::spawn_local(async move {
                    use wasm_bindgen_futures::JsFuture;

                    match web_sys::Notification::request_permission() {
                        Ok(promise) => { let _ = JsFuture::from(promise).await; }
                        Err(_) => {}
                    }

                    let perm = web_sys::Notification::permission();
                    if perm == web_sys::NotificationPermission::Granted {
                        match crate::components::notification_setup::register_and_subscribe_from_settings().await {
                            Ok(()) => {
                                set_permission_status.set("Enabled".into());
                                set_is_enabled.set(true);
                            }
                            Err(e) => {
                                set_permission_status.set(format!("Failed: {}", e));
                                set_is_enabled.set(false);
                            }
                        }
                    } else {
                        set_permission_status.set("Denied (change in browser settings)".into());
                    }
                });
            }
        }
    };

    let (test_result, set_test_result) = signal::<Option<Result<String, String>>>(None);
    let send_test = move |_| {
        set_is_testing.set(true);
        set_test_result.set(None);
        leptos::task::spawn_local(async move {
            match crate::server_fns::alerts::send_test_push().await {
                Ok(msg) => set_test_result.set(Some(Ok(msg))),
                Err(e) => set_test_result.set(Some(Err(e.to_string()))),
            }
            set_is_testing.set(false);
        });
    };

    view! {
        <div class="flex flex-col gap-3">
            <div class="flex justify-between items-center">
                <div>
                    <div class="text-sm font-medium text-stone-700 dark:text-stone-300">"Push notifications for care alerts"</div>
                    <div class="text-xs text-stone-400">{move || permission_status.get()}</div>
                </div>
                <button
                    class=move || if is_enabled.get() {
                        "relative w-11 h-6 bg-primary rounded-full transition-colors cursor-pointer border-none"
                    } else {
                        "relative w-11 h-6 bg-stone-300 dark:bg-stone-600 rounded-full transition-colors cursor-pointer border-none"
                    }
                    on:click=toggle_notifications
                >
                    <span class=move || if is_enabled.get() {
                        "absolute top-0.5 left-5.5 w-5 h-5 bg-white rounded-full transition-all shadow-sm"
                    } else {
                        "absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-all shadow-sm"
                    }></span>
                </button>
            </div>
            {move || is_enabled.get().then(|| {
                view! {
                    <div class="flex flex-col gap-2">
                        <button
                            class=format!("{} text-stone-500 bg-stone-100 hover:bg-stone-200 dark:text-stone-400 dark:bg-stone-800 dark:hover:bg-stone-700", BTN_SM)
                            disabled=move || is_testing.get()
                            on:click=send_test
                        >{move || if is_testing.get() { "Sending..." } else { "Send Test Notification" }}</button>
                        {move || test_result.get().map(|result| match result {
                            Ok(msg) => view! {
                                <div class="p-2 text-xs text-emerald-700 bg-emerald-50 rounded-lg dark:text-emerald-300 dark:bg-emerald-900/20">{msg}</div>
                            }.into_any(),
                            Err(msg) => view! {
                                <div class="p-2 text-xs text-red-700 bg-red-50 rounded-lg dark:text-red-300 dark:bg-red-900/20">{msg}</div>
                            }.into_any(),
                        })}
                    </div>
                }.into_any()
            })}
        </div>
    }.into_any()
}

/// Unsubscribe the browser's PushManager subscription.
#[cfg(feature = "hydrate")]
async fn unsubscribe_browser_push() {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let Some(window) = web_sys::window() else { return };
    let sw = window.navigator().service_worker();

    let ready = match JsFuture::from(sw.ready().unwrap_or_else(|_| {
        js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL)
    })).await {
        Ok(val) => val.dyn_into::<web_sys::ServiceWorkerRegistration>().ok(),
        Err(_) => None,
    };

    let Some(registration) = ready else { return };

    let push_manager = match registration.push_manager() {
        Ok(pm) => pm,
        Err(_) => return,
    };

    let subscription = match push_manager.get_subscription() {
        Ok(promise) => {
            match JsFuture::from(promise).await {
                Ok(val) => val.dyn_into::<web_sys::PushSubscription>().ok(),
                Err(_) => None,
            }
        }
        Err(_) => None,
    };

    if let Some(sub) = subscription {
        match sub.unsubscribe() {
            Ok(promise) => { let _ = JsFuture::from(promise).await; }
            Err(_) => {}
        }
    }
}
