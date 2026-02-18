use leptos::prelude::*;
use crate::orchid::GrowingZone;
use super::{MODAL_OVERLAY, MODAL_CONTENT, MODAL_HEADER, BTN_PRIMARY, BTN_CLOSE, BTN_SECONDARY, BTN_DANGER};

const INPUT_SM: &str = "w-full px-3 py-2 text-sm bg-white/80 border border-stone-300/50 rounded-lg outline-none transition-all duration-200 placeholder:text-stone-400 focus:bg-white focus:border-primary/40 focus:ring-2 focus:ring-primary/10 dark:bg-stone-800/80 dark:border-stone-600/50 dark:placeholder:text-stone-500 dark:focus:bg-stone-800 dark:focus:border-primary-light/40 dark:focus:ring-primary-light/10";
const LABEL_SM: &str = "block mb-1 text-xs font-semibold tracking-wider uppercase text-stone-400 dark:text-stone-500";
const BTN_SM: &str = "py-1.5 px-3 text-xs font-semibold rounded-lg border-none cursor-pointer transition-colors";

#[component]
pub fn SettingsModal(
    zones: Vec<GrowingZone>,
    on_close: impl Fn(String) + 'static + Copy + Send + Sync,
    on_zones_changed: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let (temp_unit, set_temp_unit) = signal("C".to_string());

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
                        <label>"Temperature Unit:"</label>
                        <select
                            on:change=move |ev| set_temp_unit.set(event_target_value(&ev))
                            prop:value=temp_unit
                        >
                            <option value="C">"Celsius (C)"</option>
                            <option value="F">"Fahrenheit (F)"</option>
                        </select>
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
                                    view! { <ZoneCard zone=zone on_delete=delete_zone on_zones_changed=on_zones_changed is_saving=is_zone_saving set_local_zones=set_local_zones /> }
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

                    <div class="mt-6">
                        <button class=BTN_PRIMARY on:click=move |_| on_close(temp_unit.get_untracked())>"Save Settings"</button>
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
) -> impl IntoView {
    let zone_id_for_delete = zone.id.clone();
    let zone_id_for_config = zone.id.clone();

    let light_class = match zone.light_level {
        crate::orchid::LightRequirement::High => "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-300",
        crate::orchid::LightRequirement::Low => "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300",
        _ => "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300",
    };
    let loc_class = match zone.location_type {
        crate::orchid::LocationType::Outdoor => "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-sky-100 text-sky-700 dark:bg-sky-900/30 dark:text-sky-300",
        _ => "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-stone-100 text-stone-600 dark:bg-stone-800 dark:text-stone-400",
    };

    let has_source = zone.data_source_type.is_some();
    let source_dot_class = if has_source {
        "inline-block w-2 h-2 rounded-full bg-emerald-500"
    } else {
        ""
    };

    let (show_config, set_show_config) = signal(false);

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
                <div class="flex gap-2">
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

            {move || show_config.get().then(|| {
                view! {
                    <DataSourceConfig
                        zone_id=zone_id_for_config.clone()
                        current_type=zone.data_source_type.clone()
                        current_config=zone.data_source_config.clone()
                        on_saved=move || {
                            on_zones_changed();
                        }
                        set_local_zones=set_local_zones
                    />
                }
            })}
        </div>
    }
}

/// Data source configuration form for a single zone
#[component]
fn DataSourceConfig(
    zone_id: String,
    current_type: Option<String>,
    current_config: String,
    on_saved: impl Fn() + 'static + Copy + Send + Sync,
    set_local_zones: WriteSignal<Vec<GrowingZone>>,
) -> impl IntoView {
    let initial_provider = current_type.clone().unwrap_or_default();
    let (provider, set_provider) = signal(initial_provider);

    // Parse existing config to initialize fields with correct values
    let parsed = serde_json::from_str::<serde_json::Value>(&current_config).ok();

    let get_str = |key: &str| -> String {
        parsed.as_ref()
            .and_then(|j| j.get(key))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    };

    // Tempest fields
    let (tempest_station, set_tempest_station) = signal(get_str("station_id"));
    let (tempest_token, set_tempest_token) = signal(get_str("token"));

    // AC Infinity fields
    let (aci_email, set_aci_email) = signal(get_str("email"));
    let (aci_password, set_aci_password) = signal(get_str("password"));
    let (aci_device, set_aci_device) = signal(get_str("device_id"));
    let init_port = parsed.as_ref()
        .and_then(|j| j.get("port"))
        .and_then(|v| v.as_u64())
        .map(|n| n.to_string())
        .unwrap_or_else(|| "1".to_string());
    let (aci_port, set_aci_port) = signal(init_port);

    let (test_result, set_test_result) = signal::<Option<Result<String, String>>>(None);
    let (is_testing, set_is_testing) = signal(false);
    let (is_saving_ds, set_is_saving_ds) = signal(false);

    let build_config_json = move || -> String {
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
            _ => String::new(),
        }
    };

    let test_connection = move |_| {
        let prov = provider.get();
        if prov.is_empty() { return; }
        let config = build_config_json();
        set_is_testing.set(true);
        set_test_result.set(None);

        leptos::task::spawn_local(async move {
            match crate::server_fns::climate::test_data_source(prov, config).await {
                Ok(msg) => set_test_result.set(Some(Ok(msg))),
                Err(e) => set_test_result.set(Some(Err(e.to_string()))),
            }
            set_is_testing.set(false);
        });
    };

    let zone_id_save = StoredValue::new(zone_id.clone());
    let do_save = move || {
        let prov = provider.get();
        let provider_opt = if prov.is_empty() { None } else { Some(prov.clone()) };
        let config = build_config_json();
        let zid = zone_id_save.get_value();
        set_is_saving_ds.set(true);

        leptos::task::spawn_local(async move {
            match crate::server_fns::climate::configure_zone_data_source(
                zid.clone(), provider_opt.clone(), config,
            ).await {
                Ok(()) => {
                    set_local_zones.update(|zones| {
                        if let Some(z) = zones.iter_mut().find(|z| z.id == zid) {
                            z.data_source_type = provider_opt;
                            z.data_source_config = String::new();
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
    };

    view! {
        <div class="p-3 pt-0 border-t border-stone-200/40 dark:border-stone-700/40">
            <div class="mt-3 mb-3">
                <label class=LABEL_SM>"Data Source"</label>
                <select class=INPUT_SM
                    prop:value=provider
                    on:change=move |ev| {
                        set_provider.set(event_target_value(&ev));
                        set_test_result.set(None);
                    }
                >
                    <option value="">"None"</option>
                    <option value="tempest">"Tempest Weather Station"</option>
                    <option value="ac_infinity">"AC Infinity Controller"</option>
                </select>
            </div>

            {move || match provider.get().as_str() {
                "tempest" => view! {
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
                }.into_any(),
                "ac_infinity" => view! {
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
                }.into_any(),
                _ => view! {
                    <p class="mb-3 text-xs text-stone-400 dark:text-stone-500">"No data source configured for this zone."</p>
                }.into_any(),
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
                if prov.is_empty() && current_type.is_some() {
                    // Provider changed to None — show save to remove
                    view! {
                        <div class="flex gap-2">
                            <button
                                class=format!("{} text-white bg-primary hover:bg-primary-dark", BTN_SM)
                                disabled=move || is_saving_ds.get()
                                on:click=move |_| do_save()
                            >"Remove Data Source"</button>
                        </div>
                    }.into_any()
                } else if !prov.is_empty() {
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
