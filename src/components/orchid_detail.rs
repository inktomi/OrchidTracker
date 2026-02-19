use leptos::prelude::*;
use crate::orchid::{Orchid, LightRequirement, GrowingZone, ClimateReading};
use crate::components::habitat_weather::HabitatWeatherCard;
use chrono::Local;
use super::{MODAL_OVERLAY, MODAL_CONTENT, MODAL_HEADER, BTN_PRIMARY, BTN_SECONDARY, BTN_CLOSE};

const EDIT_BTN: &str = "py-2 px-3 text-sm font-semibold text-white rounded-lg border-none cursor-pointer bg-accent hover:bg-accent-dark transition-colors";

fn light_req_to_key(lr: &LightRequirement) -> String {
    match lr {
        LightRequirement::Low => "Low".to_string(),
        LightRequirement::Medium => "Medium".to_string(),
        LightRequirement::High => "High".to_string(),
    }
}

#[component]
pub fn OrchidDetail(
    orchid: Orchid,
    zones: Vec<GrowingZone>,
    climate_readings: Vec<ClimateReading>,
    on_close: impl Fn() + 'static + Send + Sync,
    on_update: impl Fn(Orchid) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let (note, set_note) = signal(String::new());
    let (orchid_signal, set_orchid_signal) = signal(orchid.clone());
    let (is_syncing, set_is_syncing) = signal(false);

    // Edit mode state
    let (is_editing, set_is_editing) = signal(false);
    let (edit_name, set_edit_name) = signal(orchid.name.clone());
    let (edit_species, set_edit_species) = signal(orchid.species.clone());
    let (edit_water_freq, set_edit_water_freq) = signal(orchid.water_frequency_days.to_string());
    let (edit_light_req, set_edit_light_req) = signal(light_req_to_key(&orchid.light_requirement));
    let (edit_placement, set_edit_placement) = signal(orchid.placement.clone());
    let (edit_light_lux, set_edit_light_lux) = signal(orchid.light_lux.clone());
    let (edit_temp_range, set_edit_temp_range) = signal(orchid.temperature_range.clone());
    let (edit_notes, set_edit_notes) = signal(orchid.notes.clone());
    let (edit_conservation, set_edit_conservation) = signal(orchid.conservation_status.clone().unwrap_or_default());
    let (edit_temp_min, set_edit_temp_min) = signal(orchid.temp_min.map(|v| v.to_string()).unwrap_or_default());
    let (edit_temp_max, set_edit_temp_max) = signal(orchid.temp_max.map(|v| v.to_string()).unwrap_or_default());
    let (edit_humidity_min, set_edit_humidity_min) = signal(orchid.humidity_min.map(|v| v.to_string()).unwrap_or_default());
    let (edit_humidity_max, set_edit_humidity_max) = signal(orchid.humidity_max.map(|v| v.to_string()).unwrap_or_default());
    let (is_watering, set_is_watering) = signal(false);

    let zones_for_edit = zones;

    // Habitat weather: find the matching zone reading for comparison
    let habitat_zone_reading = {
        let placement = orchid.placement.clone();
        climate_readings.into_iter().find(|r| r.zone_name == placement)
    };
    let native_region = orchid.native_region.clone();
    let native_lat = orchid.native_latitude;
    let native_lon = orchid.native_longitude;

    let format_date = |dt: chrono::DateTime<chrono::Utc>| {
        dt.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string()
    };

    let on_submit_log = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_is_syncing.set(true);

        let current_note = note.get();
        let orchid_id = orchid_signal.get().id.clone();

        leptos::task::spawn_local(async move {
            let _ = crate::server_fns::orchids::add_log_entry(
                orchid_id,
                current_note,
                None,
            ).await;

            set_is_syncing.set(false);
            set_note.set(String::new());
        });
    };

    let on_edit_save = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let current = orchid_signal.get();

        let light_req = match edit_light_req.get().as_str() {
            "Low" => LightRequirement::Low,
            "High" => LightRequirement::High,
            _ => LightRequirement::Medium,
        };

        let cons = edit_conservation.get();
        let conservation_opt = if cons.is_empty() { None } else { Some(cons) };

        let updated = Orchid {
            id: current.id,
            name: edit_name.get(),
            species: edit_species.get(),
            water_frequency_days: edit_water_freq.get().parse().unwrap_or(7),
            light_requirement: light_req,
            notes: edit_notes.get(),
            placement: edit_placement.get(),
            light_lux: edit_light_lux.get(),
            temperature_range: edit_temp_range.get(),
            conservation_status: conservation_opt,
            native_region: current.native_region,
            native_latitude: current.native_latitude,
            native_longitude: current.native_longitude,
            last_watered_at: current.last_watered_at,
            temp_min: edit_temp_min.get().parse().ok(),
            temp_max: edit_temp_max.get().parse().ok(),
            humidity_min: edit_humidity_min.get().parse().ok(),
            humidity_max: edit_humidity_max.get().parse().ok(),
            history: current.history,
        };

        set_orchid_signal.set(updated.clone());
        on_update(updated);
        set_is_editing.set(false);
    };

    let on_edit_cancel = move |_| {
        let current = orchid_signal.get();
        set_edit_name.set(current.name);
        set_edit_species.set(current.species);
        set_edit_water_freq.set(current.water_frequency_days.to_string());
        set_edit_light_req.set(light_req_to_key(&current.light_requirement));
        set_edit_placement.set(current.placement);
        set_edit_light_lux.set(current.light_lux);
        set_edit_temp_range.set(current.temperature_range);
        set_edit_notes.set(current.notes);
        set_edit_conservation.set(current.conservation_status.unwrap_or_default());
        set_edit_temp_min.set(current.temp_min.map(|v| v.to_string()).unwrap_or_default());
        set_edit_temp_max.set(current.temp_max.map(|v| v.to_string()).unwrap_or_default());
        set_edit_humidity_min.set(current.humidity_min.map(|v| v.to_string()).unwrap_or_default());
        set_edit_humidity_max.set(current.humidity_max.map(|v| v.to_string()).unwrap_or_default());
        set_is_editing.set(false);
    };

    view! {
        <div class=MODAL_OVERLAY>
            <div class=MODAL_CONTENT>
                <div class=MODAL_HEADER>
                    <h2 class="m-0">{move || orchid_signal.get().name}</h2>
                    <div class="flex gap-2">
                        {move || (!is_editing.get()).then(|| {
                            view! {
                                <button class=EDIT_BTN
                                    on:click=move |_| {
                                        let current = orchid_signal.get();
                                        set_edit_name.set(current.name);
                                        set_edit_species.set(current.species);
                                        set_edit_water_freq.set(current.water_frequency_days.to_string());
                                        set_edit_light_req.set(light_req_to_key(&current.light_requirement));
                                        set_edit_placement.set(current.placement);
                                        set_edit_light_lux.set(current.light_lux);
                                        set_edit_temp_range.set(current.temperature_range);
                                        set_edit_notes.set(current.notes);
                                        set_edit_conservation.set(current.conservation_status.unwrap_or_default());
                                        set_edit_temp_min.set(current.temp_min.map(|v| v.to_string()).unwrap_or_default());
                                        set_edit_temp_max.set(current.temp_max.map(|v| v.to_string()).unwrap_or_default());
                                        set_edit_humidity_min.set(current.humidity_min.map(|v| v.to_string()).unwrap_or_default());
                                        set_edit_humidity_max.set(current.humidity_max.map(|v| v.to_string()).unwrap_or_default());
                                        set_is_editing.set(true);
                                    }
                                >"Edit"</button>
                            }
                        })}
                        <button class=BTN_CLOSE on:click=move |_| on_close()>"Close"</button>
                    </div>
                </div>
                <div>
                    {move || {
                        let zones_ref = zones_for_edit.clone();
                        if is_editing.get() {
                            view! {
                                <div class="mb-6">
                                    <form on:submit=on_edit_save>
                                        <div class="mb-4">
                                            <label>"Name:"</label>
                                            <input type="text"
                                                prop:value=edit_name
                                                on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                                required
                                            />
                                        </div>
                                        <div class="mb-4">
                                            <label>"Species:"</label>
                                            <input type="text"
                                                prop:value=edit_species
                                                on:input=move |ev| set_edit_species.set(event_target_value(&ev))
                                                required
                                            />
                                        </div>
                                        <div class="mb-4">
                                            <label>"Conservation Status:"</label>
                                            <input type="text"
                                                prop:value=edit_conservation
                                                on:input=move |ev| set_edit_conservation.set(event_target_value(&ev))
                                                placeholder="e.g. CITES II (optional)"
                                            />
                                        </div>
                                        <div class="flex flex-col gap-4 mb-4 sm:flex-row">
                                            <div class="flex-1">
                                                <label>"Water Freq (days):"</label>
                                                <input type="number"
                                                    prop:value=edit_water_freq
                                                    on:input=move |ev| set_edit_water_freq.set(event_target_value(&ev))
                                                    required
                                                />
                                            </div>
                                            <div class="flex-1">
                                                <label>"Light Req:"</label>
                                                <select
                                                    prop:value=edit_light_req
                                                    on:change=move |ev| set_edit_light_req.set(event_target_value(&ev))
                                                >
                                                    <option value="Low">"Low"</option>
                                                    <option value="Medium">"Medium"</option>
                                                    <option value="High">"High"</option>
                                                </select>
                                            </div>
                                        </div>
                                        <div class="flex flex-col gap-4 mb-4 sm:flex-row">
                                            <div class="flex-1">
                                                <label>"Zone:"</label>
                                                <select
                                                    prop:value=edit_placement
                                                    on:change=move |ev| set_edit_placement.set(event_target_value(&ev))
                                                >
                                                    {zones_ref.iter().map(|zone| {
                                                        let name = zone.name.clone();
                                                        let label = format!("{} ({})", zone.name, zone.light_level);
                                                        view! {
                                                            <option value=name>{label}</option>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </select>
                                            </div>
                                            <div class="flex-1">
                                                <label>"Light (Lux):"</label>
                                                <input type="text"
                                                    prop:value=edit_light_lux
                                                    on:input=move |ev| set_edit_light_lux.set(event_target_value(&ev))
                                                    placeholder="e.g. 5000"
                                                />
                                            </div>
                                        </div>
                                        <div class="mb-4">
                                            <label>"Temp Range:"</label>
                                            <input type="text"
                                                prop:value=edit_temp_range
                                                on:input=move |ev| set_edit_temp_range.set(event_target_value(&ev))
                                                placeholder="e.g. 18-28C"
                                            />
                                        </div>
                                        <div class="flex flex-col gap-4 mb-4 sm:flex-row">
                                            <div class="flex-1">
                                                <label>"Min Temp (C):"</label>
                                                <input type="number" step="0.1"
                                                    prop:value=edit_temp_min
                                                    on:input=move |ev| set_edit_temp_min.set(event_target_value(&ev))
                                                    placeholder="e.g. 18"
                                                />
                                            </div>
                                            <div class="flex-1">
                                                <label>"Max Temp (C):"</label>
                                                <input type="number" step="0.1"
                                                    prop:value=edit_temp_max
                                                    on:input=move |ev| set_edit_temp_max.set(event_target_value(&ev))
                                                    placeholder="e.g. 28"
                                                />
                                            </div>
                                        </div>
                                        <div class="flex flex-col gap-4 mb-4 sm:flex-row">
                                            <div class="flex-1">
                                                <label>"Min Humidity (%):"</label>
                                                <input type="number" step="0.1"
                                                    prop:value=edit_humidity_min
                                                    on:input=move |ev| set_edit_humidity_min.set(event_target_value(&ev))
                                                    placeholder="e.g. 50"
                                                />
                                            </div>
                                            <div class="flex-1">
                                                <label>"Max Humidity (%):"</label>
                                                <input type="number" step="0.1"
                                                    prop:value=edit_humidity_max
                                                    on:input=move |ev| set_edit_humidity_max.set(event_target_value(&ev))
                                                    placeholder="e.g. 80"
                                                />
                                            </div>
                                        </div>
                                        <div class="mb-4">
                                            <label>"Notes:"</label>
                                            <textarea
                                                prop:value=edit_notes
                                                on:input=move |ev| set_edit_notes.set(event_target_value(&ev))
                                                rows="3"
                                            ></textarea>
                                        </div>
                                        <div class="flex gap-2">
                                            <button type="submit" class=BTN_PRIMARY>"Save"</button>
                                            <button type="button" class=BTN_SECONDARY on:click=on_edit_cancel>"Cancel"</button>
                                        </div>
                                    </form>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="mb-4">
                                    <p class="text-sm"><strong class="text-stone-500">"Species: "</strong> <span class="italic">{move || orchid_signal.get().species}</span></p>
                                    {move || orchid_signal.get().conservation_status.map(|status| {
                                        view! { <p class="my-1 text-sm"><span class="inline-block py-0.5 px-2 text-xs font-medium rounded-full border text-danger bg-danger/5 border-danger/20">{status}</span></p> }
                                    })}
                                    <p class="text-sm"><strong class="text-stone-500">"Notes: "</strong> {move || orchid_signal.get().notes}</p>
                                </div>
                            }.into_any()
                        }
                    }}

                    {native_lat.zip(native_lon).map(|(lat, lon)| {
                        let region = native_region.clone().unwrap_or_else(|| "Native habitat".to_string());
                        let zr = habitat_zone_reading.clone();
                        view! {
                            <HabitatWeatherCard
                                native_region=region
                                latitude=lat
                                longitude=lon
                                zone_reading=zr
                            />
                        }
                    })}

                    // Watering status + Water Now button
                    <div class="flex gap-3 justify-between items-center p-4 mb-4 rounded-xl bg-secondary">
                        <div>
                            <div class="text-xs tracking-wide text-stone-400">"Watering Status"</div>
                            <div class="text-sm font-medium text-stone-700 dark:text-stone-300">
                                {move || {
                                    let o = orchid_signal.get();
                                    match o.days_until_due() {
                                        Some(days) if days < 0 => format!("Overdue by {} days", -days),
                                        Some(0) => "Due today".to_string(),
                                        Some(1) => "Due tomorrow".to_string(),
                                        Some(days) => format!("Due in {} days", days),
                                        None => "Never watered".to_string(),
                                    }
                                }}
                            </div>
                        </div>
                        <button
                            class=BTN_PRIMARY
                            disabled=move || is_watering.get()
                            on:click=move |_| {
                                set_is_watering.set(true);
                                let orchid_id = orchid_signal.get().id.clone();
                                leptos::task::spawn_local(async move {
                                    match crate::server_fns::orchids::mark_watered(orchid_id).await {
                                        Ok(updated) => set_orchid_signal.set(updated),
                                        Err(e) => log::error!("Failed to mark watered: {}", e),
                                    }
                                    set_is_watering.set(false);
                                });
                            }
                        >
                            {move || if is_watering.get() { "Watering..." } else { "Water Now" }}
                        </button>
                    </div>

                    <hr class="my-6 border-stone-200 dark:border-stone-700" />

                    <div class="mb-6">
                        <h3 class="mt-0 mb-3">"Add Entry"</h3>
                        <form on:submit=on_submit_log>
                            <div class="mb-4">
                                <label>"Note:"</label>
                                <textarea
                                    prop:value=note
                                    on:input=move |ev| set_note.set(event_target_value(&ev))
                                    placeholder="Growth update, watering note, etc."
                                    rows="3"
                                ></textarea>
                            </div>
                            <button type="submit" class=BTN_PRIMARY disabled=move || is_syncing.get()>
                                {move || if is_syncing.get() { "Saving..." } else { "Add Entry" }}
                            </button>
                        </form>
                    </div>

                    <hr class="my-6 border-stone-200 dark:border-stone-700" />

                    <div>
                        <h3 class="mt-0 mb-3">"History"</h3>
                        <div class="pl-4 mt-4 border-l-2 border-primary-light">
                            <For
                                each=move || {
                                    let mut history = orchid_signal.get().history.clone();
                                    history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                                    history
                                }
                                key=|entry| entry.id.clone()
                                children=move |entry| {
                                    let img = entry.image_filename.clone();
                                    view! {
                                        <div class="relative mb-6 before:content-[''] before:absolute before:-left-[1.4rem] before:top-[0.2rem] before:w-2.5 before:h-2.5 before:rounded-full before:bg-primary-light">
                                            <span class="block mb-1 text-xs font-medium text-stone-400">{format_date(entry.timestamp)}</span>
                                            <p class="my-1 text-sm text-stone-700 dark:text-stone-300">{entry.note.clone()}</p>
                                            {img.map(|filename| view! {
                                                <img src=format!("/images/{}", filename) class="block mt-2 max-w-full rounded-lg max-h-[300px]" alt="Orchid update" />
                                            })}
                                        </div>
                                    }
                                }
                            />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
