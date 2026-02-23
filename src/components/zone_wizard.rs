use leptos::prelude::*;
use crate::orchid::GrowingZone;
use crate::estimation::*;
use super::{MODAL_OVERLAY, MODAL_CONTENT, BTN_PRIMARY, BTN_SECONDARY};

const INPUT_WIZ: &str = "w-full px-3.5 py-2.5 text-sm bg-white/60 border border-stone-200/80 rounded-xl outline-none transition-all duration-200 placeholder:text-stone-500 focus:bg-white focus:border-primary/40 focus:ring-2 focus:ring-primary/10 dark:bg-stone-800/60 dark:border-stone-600/60 dark:placeholder:text-stone-400 dark:focus:bg-stone-800 dark:focus:border-primary-light/40 dark:focus:ring-primary-light/10";
const LABEL_WIZ: &str = "block mb-1.5 text-[11px] font-semibold tracking-wider uppercase text-stone-600 dark:text-stone-400";
const RADIO_OPTION: &str = "flex gap-2.5 items-center p-3 rounded-xl border transition-all cursor-pointer border-stone-200/80 dark:border-stone-700 hover:border-primary/30 dark:hover:border-primary-light/30 wizard-option bg-white/40 dark:bg-stone-800/30";
const RADIO_OPTION_SELECTED: &str = "flex gap-2.5 items-center p-3 rounded-xl border-2 transition-all cursor-pointer border-primary bg-primary/5 dark:border-primary-light dark:bg-primary-light/10 shadow-sm";
const CHECK_OPTION: &str = "flex gap-2.5 items-center p-2.5 rounded-xl border transition-all cursor-pointer border-stone-200/80 dark:border-stone-700 hover:border-primary/30 wizard-option bg-white/40 dark:bg-stone-800/30";
const CHECK_OPTION_SELECTED: &str = "flex gap-2.5 items-center p-2.5 rounded-xl border-2 transition-all cursor-pointer border-primary bg-primary/5 dark:border-primary-light dark:bg-primary-light/10 shadow-sm";

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

/// Labeled step progress bar with connecting track line.
#[component]
fn WizardProgress(current: ReadSignal<usize>, labels: Vec<String>) -> impl IntoView {
    let len = labels.len();
    let labels = StoredValue::new(labels);
    view! {
        <div class="flex justify-between items-start px-2 mb-8 wizard-track">
            {(0..len).map(|i| {
                let label = labels.get_value()[i].clone();
                view! {
                    <div class="flex flex-col gap-1.5 items-center min-w-0">
                        <div class=move || {
                            let cur = current.get();
                            if i == cur { "wizard-node wizard-node-active" }
                            else if i < cur { "wizard-node wizard-node-done" }
                            else { "wizard-node wizard-node-future" }
                        }>
                            {move || {
                                let cur = current.get();
                                if i < cur {
                                    "\u{2713}".to_string() // checkmark for done steps
                                } else {
                                    (i + 1).to_string()
                                }
                            }}
                        </div>
                        <span class=move || {
                            let cur = current.get();
                            if i <= cur { "text-[10px] font-bold tracking-wide text-primary dark:text-primary-light" }
                            else { "text-[10px] font-medium tracking-wide text-stone-500 dark:text-stone-400" }
                        }>{label}</span>
                    </div>
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

    let zone_name = zone.name.clone();
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
                    tracing::error!("Failed to save wizard estimation: {}", e);
                    set_is_saving.set(false);
                }
            }
        });
    };

    view! {
        <div>
            <div class="flex justify-between items-start mb-6">
                <div>
                    <h2 class="m-0 text-lg font-display">"Estimate Conditions"</h2>
                    <p class="mt-1 text-xs text-stone-500 dark:text-stone-400">{zone_name}</p>
                </div>
                <button
                    class="flex justify-center items-center w-8 h-8 rounded-full border-none transition-colors cursor-pointer text-stone-400 bg-stone-100 dark:bg-stone-800 dark:hover:bg-stone-700 dark:hover:text-stone-300 hover:bg-stone-200 hover:text-stone-600"
                    on:click=move |_| on_close()
                    aria-label="Close"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                    </svg>
                </button>
            </div>

            <WizardProgress current=step labels=vec!["Space".into(), "Light".into(), "Humidity".into(), "Review".into()] />

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

            <div class="pt-5 mt-6 border-t border-stone-200/60 dark:border-stone-700/40">
                <div class="flex gap-3 justify-between">
                    {move || if step.get() > 0 {
                        view! {
                            <button class=BTN_SECONDARY on:click=move |_| set_step.update(|s| *s -= 1)>
                                <span class="flex gap-1.5 items-center">
                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 20 20" fill="currentColor">
                                        <path fill-rule="evenodd" d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z" clip-rule="evenodd" />
                                    </svg>
                                    "Back"
                                </span>
                            </button>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }}
                    {move || if step.get() < 3 {
                        view! {
                            <button class=BTN_PRIMARY on:click=move |_| set_step.update(|s| *s += 1)>
                                <span class="flex gap-1.5 items-center">
                                    "Next"
                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 20 20" fill="currentColor">
                                        <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd" />
                                    </svg>
                                </span>
                            </button>
                        }.into_any()
                    } else {
                        view! {
                            <button
                                class="py-2.5 px-6 text-sm font-semibold text-white rounded-xl border-none shadow-sm transition-all cursor-pointer hover:shadow-md bg-accent hover:bg-accent-dark"
                                disabled=move || is_saving.get()
                                on:click=move |_| save()
                            >{move || if is_saving.get() { "Saving..." } else { "Save Estimate" }}</button>
                        }.into_any()
                    }}
                </div>
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
            <p class="mb-4 text-xs text-stone-500">"Tell us about the room where this zone is located."</p>

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
            <p class="mb-4 text-xs text-stone-500">"Windows and grow lights affect temperature."</p>

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
            <p class="mb-4 text-xs text-stone-500">"How would you describe the air, and what humidity aids do you use?"</p>

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

/// Step 4: Review & save — dramatic estimation display with animated result cards.
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
            <p class="mb-5 text-xs text-stone-500">"Here's our estimate based on your answers."</p>

            // Estimation result cards — spring animation reveals
            <div class="flex gap-3 mb-5">
                <div class="flex-1 p-4 text-center bg-gradient-to-br rounded-2xl border from-primary/5 to-primary/10 border-primary/15 wizard-result-reveal dark:from-primary-light/5 dark:to-primary-light/10 dark:border-primary-light/15">
                    <div class="mb-1 font-bold tracking-widest uppercase text-[10px] text-stone-500 dark:text-stone-400">"Temperature"</div>
                    <div class="text-3xl font-display text-primary dark:text-primary-light">
                        {move || {
                            let est = estimation.get();
                            if is_f {
                                format!("{:.0}-{:.0}", c_to_f(est.temperature_low_c), c_to_f(est.temperature_high_c))
                            } else {
                                format!("{:.0}-{:.0}", est.temperature_low_c, est.temperature_high_c)
                            }
                        }}
                    </div>
                    <div class="text-xs font-medium text-primary/60 dark:text-primary-light/60">{if is_f { "\u{00B0}F" } else { "\u{00B0}C" }}</div>
                </div>
                <div class="flex-1 p-4 text-center bg-gradient-to-br rounded-2xl border from-accent/5 to-accent/10 border-accent/15 wizard-result-reveal wizard-result-delay-1 dark:from-accent-light/5 dark:to-accent-light/10 dark:border-accent-light/15">
                    <div class="mb-1 font-bold tracking-widest uppercase text-[10px] text-stone-500 dark:text-stone-400">"Humidity"</div>
                    <div class="text-3xl font-display text-accent dark:text-accent-light">
                        {move || format!("{:.0}", estimation.get().humidity_pct)}
                    </div>
                    <div class="text-xs font-medium text-accent/60 dark:text-accent-light/60">"%"</div>
                </div>
                {move || {
                    let est = estimation.get();
                    let vpd = crate::estimation::calculate_vpd(
                        (est.temperature_low_c + est.temperature_high_c) / 2.0,
                        est.humidity_pct,
                    );
                    view! {
                        <div class="flex-1 p-4 text-center bg-gradient-to-br rounded-2xl border from-stone-100/80 to-stone-50 border-stone-200/60 wizard-result-reveal wizard-result-delay-2 dark:from-stone-800/50 dark:to-stone-800/30 dark:border-stone-700/40">
                            <div class="mb-1 font-bold tracking-widest uppercase text-[10px] text-stone-500 dark:text-stone-400">"VPD"</div>
                            <div class="text-3xl font-display text-stone-600 dark:text-stone-300">
                                {format!("{:.2}", vpd)}
                            </div>
                            <div class="text-xs font-medium text-stone-500 dark:text-stone-400">"kPa"</div>
                        </div>
                    }
                }}
            </div>

            // Override section — visually secondary
            <div class="p-3.5 rounded-xl border border-dashed border-stone-200/80 dark:border-stone-700/60">
                <p class="mb-3 font-bold tracking-widest uppercase text-[10px] text-stone-500 dark:text-stone-400">"Fine-tune (optional)"</p>
                <div class="flex gap-3">
                    <div class="flex-1">
                        <label class=LABEL_WIZ>{if is_f { "Temperature (\u{00B0}F)" } else { "Temperature (\u{00B0}C)" }}</label>
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
    let zone_name = zone.name.clone();
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
                tracing::warn!("Geolocation failed");
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
                    tracing::error!("Failed to configure weather API: {}", e);
                    set_is_saving.set(false);
                }
            }
        });
    };

    view! {
        <div>
            <div class="flex justify-between items-start mb-6">
                <div>
                    <h2 class="m-0 text-lg font-display">"Outdoor Weather Setup"</h2>
                    <p class="mt-1 text-xs text-stone-500 dark:text-stone-400">{zone_name}</p>
                </div>
                <button
                    class="flex justify-center items-center w-8 h-8 rounded-full border-none transition-colors cursor-pointer text-stone-400 bg-stone-100 dark:bg-stone-800 dark:hover:bg-stone-700 dark:hover:text-stone-300 hover:bg-stone-200 hover:text-stone-600"
                    on:click=move |_| on_close()
                    aria-label="Close"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                    </svg>
                </button>
            </div>

            <WizardProgress current=step labels=vec!["Location".into(), "Preview".into()] />

            {move || match step.get() {
                0 => view! {
                    <div class="wizard-step-in">
                        <h3 class="mb-1 text-base font-medium text-stone-700 dark:text-stone-300">"Location"</h3>
                        <p class="mb-4 text-xs text-stone-500">"We'll fetch live weather data for this location."</p>

                        <button
                            class="flex gap-2 justify-center items-center py-3 mb-4 w-full text-sm font-semibold text-white rounded-xl border-none shadow-sm transition-all cursor-pointer hover:shadow-md bg-primary hover:bg-primary-dark"
                            disabled=move || is_locating.get()
                            on:click=get_location
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M5.05 4.05a7 7 0 119.9 9.9L10 18.9l-4.95-4.95a7 7 0 010-9.9zM10 11a2 2 0 100-4 2 2 0 000 4z" clip-rule="evenodd" />
                            </svg>
                            {move || if is_locating.get() { "Getting location..." } else { "Use My Location" }}
                        </button>

                        <p class="mb-3 text-xs text-center text-stone-500">"or enter coordinates manually:"</p>

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
                        <p class="mb-4 text-xs text-stone-500">"Test the weather feed, then save to enable automatic polling."</p>

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
