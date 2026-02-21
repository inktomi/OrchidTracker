use leptos::prelude::*;
use crate::orchid::GrowingZone;
use crate::estimation::*;
use super::{MODAL_OVERLAY, MODAL_CONTENT, BTN_PRIMARY, BTN_SECONDARY, BTN_CLOSE};

const WIZARD_STEP_DOT_ACTIVE: &str = "w-2.5 h-2.5 rounded-full bg-primary scale-125 transition-all duration-300";
const WIZARD_STEP_DOT_DONE: &str = "w-2.5 h-2.5 rounded-full bg-primary/40 transition-all duration-300";
const WIZARD_STEP_DOT_FUTURE: &str = "w-2.5 h-2.5 rounded-full bg-stone-300 dark:bg-stone-600 transition-all duration-300";

const INPUT_WIZ: &str = "w-full px-3 py-2.5 text-sm bg-white/80 border border-stone-300/50 rounded-lg outline-none transition-all duration-200 placeholder:text-stone-400 focus:bg-white focus:border-primary/40 focus:ring-2 focus:ring-primary/10 dark:bg-stone-800/80 dark:border-stone-600/50 dark:placeholder:text-stone-500 dark:focus:bg-stone-800 dark:focus:border-primary-light/40 dark:focus:ring-primary-light/10";
const LABEL_WIZ: &str = "block mb-1.5 text-xs font-semibold tracking-wider uppercase text-stone-400 dark:text-stone-500";
const RADIO_OPTION: &str = "flex gap-2 items-center p-2.5 rounded-lg border transition-colors cursor-pointer border-stone-200 dark:border-stone-700 hover:border-primary/30 dark:hover:border-primary-light/30";
const RADIO_OPTION_SELECTED: &str = "flex gap-2 items-center p-2.5 rounded-lg border transition-colors cursor-pointer border-primary bg-primary/5 dark:border-primary-light dark:bg-primary-light/10";
const CHECK_OPTION: &str = "flex gap-2 items-center p-2 rounded-lg border transition-colors cursor-pointer border-stone-200 dark:border-stone-700 hover:border-primary/30";
const CHECK_OPTION_SELECTED: &str = "flex gap-2 items-center p-2 rounded-lg border transition-colors cursor-pointer border-primary bg-primary/5 dark:border-primary-light dark:bg-primary-light/10";

/// Main wizard component — branches on indoor vs outdoor.
#[component]
pub fn ZoneConditionWizard(
    zone: GrowingZone,
    temp_unit: String,
    on_close: impl Fn() + 'static + Copy + Send + Sync,
    on_saved: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let is_outdoor = zone.location_type == crate::orchid::LocationType::Outdoor;

    view! {
        <div class=MODAL_OVERLAY>
            <div class=MODAL_CONTENT>
                {if is_outdoor {
                    view! { <OutdoorWizard zone=zone.clone() on_close=on_close on_saved=on_saved /> }.into_any()
                } else {
                    view! { <IndoorWizard zone=zone.clone() temp_unit=temp_unit on_close=on_close on_saved=on_saved /> }.into_any()
                }}
            </div>
        </div>
    }
}

/// Progress dots indicator.
#[component]
fn ProgressDots(current: ReadSignal<usize>, total: usize) -> impl IntoView {
    view! {
        <div class="flex gap-2 justify-center mb-6">
            {(0..total).map(|i| {
                view! {
                    <span class=move || {
                        let cur = current.get();
                        if i == cur { WIZARD_STEP_DOT_ACTIVE }
                        else if i < cur { WIZARD_STEP_DOT_DONE }
                        else { WIZARD_STEP_DOT_FUTURE }
                    }></span>
                }
            }).collect::<Vec<_>>()}
        </div>
    }.into_any()
}

/// Indoor wizard: 4 steps.
#[component]
fn IndoorWizard(
    zone: GrowingZone,
    temp_unit: String,
    on_close: impl Fn() + 'static + Copy + Send + Sync,
    on_saved: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let (step, set_step) = signal(0usize);
    let (is_saving, set_is_saving) = signal(false);

    // Step 1: Room & thermostat
    let (room_type, set_room_type) = signal("LivingRoom".to_string());
    let default_temp = if temp_unit == "F" { "72".to_string() } else { "22".to_string() };
    let (thermostat, set_thermostat) = signal(default_temp);
    let temp_unit_sig = StoredValue::new(temp_unit);

    // Step 2: Light & proximity
    let (has_window, set_has_window) = signal(false);
    let (window_dir, set_window_dir) = signal("South".to_string());
    let (has_grow_lights, set_has_grow_lights) = signal(false);

    // Step 3: Humidity
    let (air_desc, set_air_desc) = signal("Average".to_string());
    let (boosters, set_boosters) = signal::<Vec<String>>(vec![]);

    // Computed estimation (reactive)
    let estimation = Memo::new(move |_| {
        let room = match room_type.get().as_str() {
            "Kitchen" => RoomType::Kitchen,
            "Bathroom" => RoomType::Bathroom,
            "Bedroom" => RoomType::Bedroom,
            "Sunroom" => RoomType::Sunroom,
            "Office" => RoomType::Office,
            "Garage" => RoomType::Garage,
            "Other" => RoomType::Other,
            _ => RoomType::LivingRoom,
        };

        let temp_str = thermostat.get();
        let temp_val: f64 = temp_str.parse().unwrap_or(22.0);
        let temp_c = if temp_unit_sig.get_value() == "F" { f_to_c(temp_val) } else { temp_val };

        let win_dir = if has_window.get() {
            Some(match window_dir.get().as_str() {
                "North" => WindowDirection::North,
                "East" => WindowDirection::East,
                "West" => WindowDirection::West,
                _ => WindowDirection::South,
            })
        } else {
            None
        };

        let air = match air_desc.get().as_str() {
            "VeryDry" => AirDescription::VeryDry,
            "Humid" => AirDescription::Humid,
            _ => AirDescription::Average,
        };

        let booster_list: Vec<HumidityBooster> = boosters.get().iter().filter_map(|b| {
            match b.as_str() {
                "Humidifier" => Some(HumidityBooster::Humidifier),
                "RegularMisting" => Some(HumidityBooster::RegularMisting),
                "PebbleTray" => Some(HumidityBooster::PebbleTray),
                "GroupedPlants" => Some(HumidityBooster::GroupedPlants),
                _ => None,
            }
        }).collect();

        let input = IndoorEstimationInput {
            room_type: room,
            thermostat_c: temp_c,
            has_window: has_window.get(),
            window_direction: win_dir,
            has_grow_lights: has_grow_lights.get(),
            air_description: air,
            humidity_boosters: booster_list,
        };

        estimate_indoor(&input)
    });

    // Override signals for step 4
    let (override_temp, set_override_temp) = signal::<Option<String>>(None);
    let (override_humidity, set_override_humidity) = signal::<Option<String>>(None);

    let zone_stored = StoredValue::new(zone);

    let save = move || {
        set_is_saving.set(true);
        let est = estimation.get();
        let z = zone_stored.get_value();

        let temp = override_temp.get()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or((est.temperature_low_c + est.temperature_high_c) / 2.0);
        let hum = override_humidity.get()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(est.humidity_pct);

        leptos::task::spawn_local(async move {
            match crate::server_fns::climate::save_wizard_estimation(
                z.id.clone(), z.name.clone(), temp, hum,
            ).await {
                Ok(()) => {
                    on_saved();
                    on_close();
                }
                Err(e) => {
                    log::error!("Failed to save wizard estimation: {}", e);
                    set_is_saving.set(false);
                }
            }
        });
    };

    view! {
        <div>
            <div class="flex justify-between items-center mb-4">
                <h2 class="m-0 text-lg">"Estimate Conditions"</h2>
                <button class=BTN_CLOSE on:click=move |_| on_close()>"Close"</button>
            </div>

            <ProgressDots current=step total=4 />

            {move || match step.get() {
                0 => view! { <Step1Room room_type=room_type set_room_type=set_room_type
                    thermostat=thermostat set_thermostat=set_thermostat
                    temp_unit=temp_unit_sig.get_value() /> }.into_any(),
                1 => view! { <Step2Light has_window=has_window set_has_window=set_has_window
                    window_dir=window_dir set_window_dir=set_window_dir
                    has_grow_lights=has_grow_lights set_has_grow_lights=set_has_grow_lights /> }.into_any(),
                2 => view! { <Step3Humidity air_desc=air_desc set_air_desc=set_air_desc
                    boosters=boosters set_boosters=set_boosters /> }.into_any(),
                _ => view! { <Step4Review estimation=estimation temp_unit=temp_unit_sig.get_value()
                    override_temp=override_temp set_override_temp=set_override_temp
                    override_humidity=override_humidity set_override_humidity=set_override_humidity /> }.into_any(),
            }}

            <div class="flex gap-3 justify-between mt-6">
                {move || if step.get() > 0 {
                    view! {
                        <button class=BTN_SECONDARY on:click=move |_| set_step.update(|s| *s -= 1)>"Back"</button>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }}
                {move || if step.get() < 3 {
                    view! {
                        <button class=BTN_PRIMARY on:click=move |_| set_step.update(|s| *s += 1)>"Next"</button>
                    }.into_any()
                } else {
                    view! {
                        <button class=BTN_PRIMARY disabled=move || is_saving.get()
                            on:click=move |_| save()
                        >{move || if is_saving.get() { "Saving..." } else { "Save Estimate" }}</button>
                    }.into_any()
                }}
            </div>
        </div>
    }
}

/// Step 1: Room type + thermostat
#[component]
fn Step1Room(
    room_type: ReadSignal<String>,
    set_room_type: WriteSignal<String>,
    thermostat: ReadSignal<String>,
    set_thermostat: WriteSignal<String>,
    temp_unit: String,
) -> impl IntoView {
    let rooms = vec![
        ("LivingRoom", "Living Room"),
        ("Kitchen", "Kitchen"),
        ("Bathroom", "Bathroom"),
        ("Bedroom", "Bedroom"),
        ("Sunroom", "Sunroom"),
        ("Office", "Office"),
        ("Garage", "Garage"),
        ("Other", "Other"),
    ];

    view! {
        <div class="wizard-step-in">
            <h3 class="mb-1 text-base font-medium text-stone-700 dark:text-stone-300">"Your Space"</h3>
            <p class="mb-4 text-xs text-stone-400">"Tell us about the room where this zone is located."</p>

            <div class="mb-4">
                <label class=LABEL_WIZ>"Room Type"</label>
                <select class=INPUT_WIZ
                    prop:value=room_type
                    on:change=move |ev| set_room_type.set(event_target_value(&ev))
                >
                    {rooms.into_iter().map(|(val, label)| {
                        view! { <option value=val>{label}</option> }
                    }).collect::<Vec<_>>()}
                </select>
            </div>

            <div>
                <label class=LABEL_WIZ>{format!("Thermostat Setting ({})", if temp_unit == "F" { "°F" } else { "°C" })}</label>
                <input type="number" class=INPUT_WIZ
                    prop:value=thermostat
                    on:input=move |ev| set_thermostat.set(event_target_value(&ev))
                    step="1"
                />
            </div>
        </div>
    }.into_any()
}

/// Step 2: Window + grow lights
#[component]
fn Step2Light(
    has_window: ReadSignal<bool>,
    set_has_window: WriteSignal<bool>,
    window_dir: ReadSignal<String>,
    set_window_dir: WriteSignal<String>,
    has_grow_lights: ReadSignal<bool>,
    set_has_grow_lights: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="wizard-step-in">
            <h3 class="mb-1 text-base font-medium text-stone-700 dark:text-stone-300">"Light & Proximity"</h3>
            <p class="mb-4 text-xs text-stone-400">"Windows and grow lights affect temperature."</p>

            <div class="mb-4">
                <label class=LABEL_WIZ>"Near a Window?"</label>
                <div class="flex gap-3">
                    <button
                        class=move || if has_window.get() { RADIO_OPTION_SELECTED } else { RADIO_OPTION }
                        on:click=move |_| set_has_window.set(true)
                    >"Yes"</button>
                    <button
                        class=move || if !has_window.get() { RADIO_OPTION_SELECTED } else { RADIO_OPTION }
                        on:click=move |_| set_has_window.set(false)
                    >"No"</button>
                </div>
            </div>

            {move || has_window.get().then(|| {
                let directions = vec![("North", "North"), ("South", "South"), ("East", "East"), ("West", "West")];
                view! {
                    <div class="mb-4">
                        <label class=LABEL_WIZ>"Window Direction"</label>
                        <div class="flex flex-wrap gap-2">
                            {directions.into_iter().map(|(val, label)| {
                                let val_owned = val.to_string();
                                view! {
                                    <button
                                        class=move || if window_dir.get() == val { RADIO_OPTION_SELECTED } else { RADIO_OPTION }
                                        on:click=move |_| set_window_dir.set(val_owned.clone())
                                    >{label}</button>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                }
            })}

            <div>
                <label class=LABEL_WIZ>"Grow Lights?"</label>
                <div class="flex gap-3">
                    <button
                        class=move || if has_grow_lights.get() { RADIO_OPTION_SELECTED } else { RADIO_OPTION }
                        on:click=move |_| set_has_grow_lights.set(true)
                    >"Yes"</button>
                    <button
                        class=move || if !has_grow_lights.get() { RADIO_OPTION_SELECTED } else { RADIO_OPTION }
                        on:click=move |_| set_has_grow_lights.set(false)
                    >"No"</button>
                </div>
            </div>
        </div>
    }.into_any()
}

/// Step 3: Humidity
#[component]
fn Step3Humidity(
    air_desc: ReadSignal<String>,
    set_air_desc: WriteSignal<String>,
    boosters: ReadSignal<Vec<String>>,
    set_boosters: WriteSignal<Vec<String>>,
) -> impl IntoView {
    let toggle_booster = move |name: &str| {
        let n = name.to_string();
        set_boosters.update(move |list| {
            if list.contains(&n) {
                list.retain(|b| b != &n);
            } else {
                list.push(n.clone());
            }
        });
    };

    let booster_items = vec![
        ("Humidifier", "Humidifier"),
        ("RegularMisting", "Regular Misting"),
        ("PebbleTray", "Pebble Tray"),
        ("GroupedPlants", "Grouped Plants"),
    ];

    view! {
        <div class="wizard-step-in">
            <h3 class="mb-1 text-base font-medium text-stone-700 dark:text-stone-300">"Humidity"</h3>
            <p class="mb-4 text-xs text-stone-400">"How would you describe the air, and what humidity aids do you use?"</p>

            <div class="mb-4">
                <label class=LABEL_WIZ>"Air Description"</label>
                <div class="flex flex-wrap gap-2">
                    {vec![("VeryDry", "Very Dry"), ("Average", "Average"), ("Humid", "Humid")]
                        .into_iter().map(|(val, label)| {
                            let val_owned = val.to_string();
                            view! {
                                <button
                                    class=move || if air_desc.get() == val { RADIO_OPTION_SELECTED } else { RADIO_OPTION }
                                    on:click=move |_| set_air_desc.set(val_owned.clone())
                                >{label}</button>
                            }
                        }).collect::<Vec<_>>()}
                </div>
            </div>

            <div>
                <label class=LABEL_WIZ>"Humidity Boosters (select all that apply)"</label>
                <div class="flex flex-col gap-2">
                    {booster_items.into_iter().map(|(val, label)| {
                        let val_str = val.to_string();
                        let val_check = val.to_string();
                        view! {
                            <button
                                class=move || if boosters.get().contains(&val_check) { CHECK_OPTION_SELECTED } else { CHECK_OPTION }
                                on:click=move |_| toggle_booster(&val_str)
                            >
                                <span class="text-sm">{label}</span>
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </div>
    }.into_any()
}

/// Step 4: Review & save
#[component]
fn Step4Review(
    estimation: Memo<EstimationResult>,
    temp_unit: String,
    override_temp: ReadSignal<Option<String>>,
    set_override_temp: WriteSignal<Option<String>>,
    override_humidity: ReadSignal<Option<String>>,
    set_override_humidity: WriteSignal<Option<String>>,
) -> impl IntoView {
    let is_f = temp_unit == "F";

    view! {
        <div class="wizard-step-in">
            <h3 class="mb-1 text-base font-medium text-stone-700 dark:text-stone-300">"Review & Save"</h3>
            <p class="mb-4 text-xs text-stone-400">"Here's our estimate. You can adjust the values before saving."</p>

            <div class="p-4 mb-4 rounded-xl border bg-primary/5 border-primary/20 dark:bg-primary-light/5 dark:border-primary-light/20">
                <div class="flex flex-wrap gap-6 justify-center">
                    <div class="flex flex-col items-center">
                        <span class="text-xs font-medium tracking-wider uppercase text-stone-400">"Temperature"</span>
                        <span class="text-2xl font-semibold text-primary dark:text-primary-light">
                            {move || {
                                let est = estimation.get();
                                if is_f {
                                    format!("{:.0}-{:.0}°F", c_to_f(est.temperature_low_c), c_to_f(est.temperature_high_c))
                                } else {
                                    format!("{:.0}-{:.0}°C", est.temperature_low_c, est.temperature_high_c)
                                }
                            }}
                        </span>
                    </div>
                    <div class="flex flex-col items-center">
                        <span class="text-xs font-medium tracking-wider uppercase text-stone-400">"Humidity"</span>
                        <span class="text-2xl font-semibold text-primary dark:text-primary-light">
                            {move || format!("{:.0}%", estimation.get().humidity_pct)}
                        </span>
                    </div>
                </div>
            </div>

            <p class="mb-3 text-xs font-medium text-stone-400">"Override (optional):"</p>
            <div class="flex gap-3">
                <div class="flex-1">
                    <label class=LABEL_WIZ>{if is_f { "Temperature (°F)" } else { "Temperature (°C)" }}</label>
                    <input type="number" class=INPUT_WIZ
                        placeholder=move || {
                            let est = estimation.get();
                            let mid = (est.temperature_low_c + est.temperature_high_c) / 2.0;
                            if is_f { format!("{:.0}", c_to_f(mid)) } else { format!("{:.0}", mid) }
                        }
                        prop:value=move || override_temp.get().unwrap_or_default()
                        on:input=move |ev| {
                            let v = event_target_value(&ev);
                            if v.is_empty() {
                                set_override_temp.set(None);
                            } else if is_f {
                                // Convert F input to C for storage
                                if let Ok(f_val) = v.parse::<f64>() {
                                    set_override_temp.set(Some(format!("{:.1}", f_to_c(f_val))));
                                }
                            } else {
                                set_override_temp.set(Some(v));
                            }
                        }
                    />
                </div>
                <div class="flex-1">
                    <label class=LABEL_WIZ>"Humidity (%)"</label>
                    <input type="number" class=INPUT_WIZ
                        placeholder=move || format!("{:.0}", estimation.get().humidity_pct)
                        prop:value=move || override_humidity.get().unwrap_or_default()
                        on:input=move |ev| {
                            let v = event_target_value(&ev);
                            set_override_humidity.set(if v.is_empty() { None } else { Some(v) });
                        }
                    />
                </div>
            </div>
        </div>
    }.into_any()
}

/// Outdoor wizard: 2 steps (location + preview/save).
#[component]
fn OutdoorWizard(
    zone: GrowingZone,
    on_close: impl Fn() + 'static + Copy + Send + Sync,
    on_saved: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let (step, set_step) = signal(0usize);
    let (latitude, set_latitude) = signal(String::new());
    let (longitude, set_longitude) = signal(String::new());
    let (is_locating, set_is_locating) = signal(false);
    let _ = &set_is_locating; // used in hydrate cfg
    let (preview, set_preview) = signal::<Option<Result<String, String>>>(None);
    let (is_testing, set_is_testing) = signal(false);
    let (is_saving, set_is_saving) = signal(false);

    let zone_stored = StoredValue::new(zone);

    // Browser geolocation
    let get_location = move |_| {
        #[cfg(feature = "hydrate")]
        {
            set_is_locating.set(true);
            use wasm_bindgen::prelude::*;

            let success = Closure::once(move |pos: JsValue| {
                let coords = js_sys::Reflect::get(&pos, &"coords".into()).unwrap_or_default();
                let lat = js_sys::Reflect::get(&coords, &"latitude".into())
                    .unwrap_or_default()
                    .as_f64()
                    .unwrap_or(0.0);
                let lon = js_sys::Reflect::get(&coords, &"longitude".into())
                    .unwrap_or_default()
                    .as_f64()
                    .unwrap_or(0.0);
                set_latitude.set(format!("{:.4}", lat));
                set_longitude.set(format!("{:.4}", lon));
                set_is_locating.set(false);
            });

            let error = Closure::once(move |_: JsValue| {
                set_is_locating.set(false);
                log::warn!("Geolocation failed");
            });

            if let Some(window) = web_sys::window() {
                // Use js_sys::Reflect to call navigator.geolocation.getCurrentPosition
                // without requiring web_sys Geolocation feature
                let nav = window.navigator();
                let geo = js_sys::Reflect::get(&nav, &"geolocation".into()).ok();
                if let Some(geo) = geo {
                    if !geo.is_undefined() && !geo.is_null() {
                        let _ = js_sys::Reflect::apply(
                            &js_sys::Function::from(
                                js_sys::Reflect::get(&geo, &"getCurrentPosition".into())
                                    .unwrap_or_default()
                            ),
                            &geo,
                            &js_sys::Array::of2(
                                success.as_ref(),
                                error.as_ref(),
                            ),
                        );
                    }
                }
            }

            success.forget();
            error.forget();
        }
    };

    let test_api = move |_| {
        let lat: f64 = latitude.get().parse().unwrap_or(0.0);
        let lon: f64 = longitude.get().parse().unwrap_or(0.0);
        if lat == 0.0 && lon == 0.0 { return; }
        set_is_testing.set(true);
        set_preview.set(None);

        leptos::task::spawn_local(async move {
            match crate::server_fns::climate::test_weather_api(lat, lon).await {
                Ok(msg) => set_preview.set(Some(Ok(msg))),
                Err(e) => set_preview.set(Some(Err(e.to_string()))),
            }
            set_is_testing.set(false);
        });
    };

    let do_save = move || {
        let lat: f64 = latitude.get().parse().unwrap_or(0.0);
        let lon: f64 = longitude.get().parse().unwrap_or(0.0);
        if lat == 0.0 && lon == 0.0 { return; }
        set_is_saving.set(true);

        let z = zone_stored.get_value();
        let config = serde_json::json!({
            "latitude": lat,
            "longitude": lon,
        }).to_string();

        leptos::task::spawn_local(async move {
            match crate::server_fns::climate::configure_zone_data_source(
                z.id.clone(), Some("weather_api".to_string()), config,
            ).await {
                Ok(()) => {
                    on_saved();
                    on_close();
                }
                Err(e) => {
                    log::error!("Failed to configure weather API: {}", e);
                    set_is_saving.set(false);
                }
            }
        });
    };

    view! {
        <div>
            <div class="flex justify-between items-center mb-4">
                <h2 class="m-0 text-lg">"Outdoor Weather Setup"</h2>
                <button class=BTN_CLOSE on:click=move |_| on_close()>"Close"</button>
            </div>

            <ProgressDots current=step total=2 />

            {move || match step.get() {
                0 => view! {
                    <div class="wizard-step-in">
                        <h3 class="mb-1 text-base font-medium text-stone-700 dark:text-stone-300">"Location"</h3>
                        <p class="mb-4 text-xs text-stone-400">"We'll fetch live weather data for this location."</p>

                        <button
                            class=format!("{} mb-4 w-full", BTN_PRIMARY)
                            disabled=move || is_locating.get()
                            on:click=get_location
                        >
                            {move || if is_locating.get() { "Getting location..." } else { "Use My Location" }}
                        </button>

                        <p class="mb-3 text-xs text-center text-stone-400">"or enter coordinates manually:"</p>

                        <div class="flex gap-3">
                            <div class="flex-1">
                                <label class=LABEL_WIZ>"Latitude"</label>
                                <input type="number" class=INPUT_WIZ step="0.0001"
                                    placeholder="e.g. 37.7749"
                                    prop:value=latitude
                                    on:input=move |ev| set_latitude.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="flex-1">
                                <label class=LABEL_WIZ>"Longitude"</label>
                                <input type="number" class=INPUT_WIZ step="0.0001"
                                    placeholder="e.g. -122.4194"
                                    prop:value=longitude
                                    on:input=move |ev| set_longitude.set(event_target_value(&ev))
                                />
                            </div>
                        </div>
                    </div>
                }.into_any(),
                _ => view! {
                    <div class="wizard-step-in">
                        <h3 class="mb-1 text-base font-medium text-stone-700 dark:text-stone-300">"Preview & Save"</h3>
                        <p class="mb-4 text-xs text-stone-400">"Test the weather feed, then save to enable automatic polling."</p>

                        <div class="p-3 mb-4 text-sm text-center rounded-lg bg-stone-50 text-stone-600 dark:bg-stone-800 dark:text-stone-400">
                            {move || format!("Coordinates: {}, {}", latitude.get(), longitude.get())}
                        </div>

                        <button
                            class=format!("{} mb-3 w-full", BTN_SECONDARY)
                            disabled=move || is_testing.get()
                            on:click=test_api
                        >
                            {move || if is_testing.get() { "Fetching..." } else { "Fetch Current Weather" }}
                        </button>

                        {move || preview.get().map(|result| match result {
                            Ok(msg) => view! {
                                <div class="p-3 mb-4 text-sm text-emerald-700 bg-emerald-50 rounded-lg dark:text-emerald-300 dark:bg-emerald-900/20">{msg}</div>
                            }.into_any(),
                            Err(msg) => view! {
                                <div class="p-3 mb-4 text-sm text-red-700 bg-red-50 rounded-lg dark:text-red-300 dark:bg-red-900/20">{msg}</div>
                            }.into_any(),
                        })}
                    </div>
                }.into_any(),
            }}

            <div class="flex gap-3 justify-between mt-6">
                {move || if step.get() > 0 {
                    view! {
                        <button class=BTN_SECONDARY on:click=move |_| set_step.set(0)>"Back"</button>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }}
                {move || if step.get() == 0 {
                    view! {
                        <button class=BTN_PRIMARY
                            disabled=move || latitude.get().is_empty() || longitude.get().is_empty()
                            on:click=move |_| { set_step.set(1); }
                        >"Next"</button>
                    }.into_any()
                } else {
                    view! {
                        <button class=BTN_PRIMARY disabled=move || is_saving.get()
                            on:click=move |_| do_save()
                        >{move || if is_saving.get() { "Saving..." } else { "Save & Enable" }}</button>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
