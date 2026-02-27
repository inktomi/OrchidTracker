use super::{BTN_CLOSE, BTN_PRIMARY, MODAL_CONTENT, MODAL_HEADER, MODAL_OVERLAY};
use crate::components::scanner::AnalysisResult;
use crate::orchid::{GrowingZone, LightRequirement, Orchid};
use leptos::prelude::*;

#[component]
pub fn AddOrchidForm(
    zones: Vec<GrowingZone>,
    on_add: impl Fn(Orchid) + 'static + Send + Sync,
    on_close: impl Fn() + 'static + Copy + Send + Sync,
    prefill_data: Memo<Option<AnalysisResult>>,
) -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (species, set_species) = signal(String::new());
    let (water_freq, set_water_freq) = signal("7".to_string());
    let (light, set_light) = signal("Medium".to_string());
    let default_placement = zones.first().map(|z| z.name.clone()).unwrap_or_default();
    let (placement, set_placement) = signal(default_placement);
    let (notes, set_notes) = signal(String::new());
    let (lux, set_lux) = signal(String::new());
    let (temp, set_temp) = signal(String::new());
    let (conservation, set_conservation) = signal(String::new());
    let (native_region, set_native_region) = signal::<Option<String>>(None);
    let (native_latitude, set_native_latitude) = signal::<Option<f64>>(None);
    let (native_longitude, set_native_longitude) = signal::<Option<f64>>(None);
    let (temp_min, set_temp_min) = signal(String::new());
    let (temp_max, set_temp_max) = signal(String::new());
    let (humidity_min, set_humidity_min) = signal(String::new());
    let (humidity_max, set_humidity_max) = signal(String::new());
    let (pot_medium, set_pot_medium) = signal(String::new());
    let (pot_size, set_pot_size) = signal(String::new());
    let (pot_type, set_pot_type) = signal(String::new());
    let (par_ppfd, set_par_ppfd) = signal(String::new());

    // Seasonal signals
    let (rest_start_month, set_rest_start_month) = signal::<Option<u32>>(None);
    let (rest_end_month, set_rest_end_month) = signal::<Option<u32>>(None);
    let (bloom_start_month, set_bloom_start_month) = signal::<Option<u32>>(None);
    let (bloom_end_month, set_bloom_end_month) = signal::<Option<u32>>(None);
    let (rest_water_mult, set_rest_water_mult) = signal::<Option<f64>>(None);
    let (rest_fert_mult, set_rest_fert_mult) = signal::<Option<f64>>(None);
    let (active_water_mult, set_active_water_mult) = signal::<Option<f64>>(None);
    let (active_fert_mult, set_active_fert_mult) = signal::<Option<f64>>(None);

    let zones_for_prefill = zones.clone();

    Effect::new(move |_| {
        if let Some(data) = prefill_data.get() {
            set_name.set(data.species_name.clone());
            set_species.set(data.species_name);
            set_water_freq.set(data.water_freq.to_string());

            let light_val = match data.light_req {
                LightRequirement::Low => "Low",
                LightRequirement::Medium => "Medium",
                LightRequirement::High => "High",
            };
            set_light.set(light_val.to_string());

            // Find best matching zone from scanner suggestion
            let suggestion = data.placement_suggestion.to_lowercase();
            let best_zone = zones_for_prefill
                .iter()
                .find(|z| z.name.to_lowercase().contains(&suggestion))
                .or_else(|| zones_for_prefill.first());

            if let Some(zone) = best_zone {
                set_placement.set(zone.name.clone());
            }

            set_temp.set(data.temp_range);

            if let Some(status) = data.conservation_status {
                set_conservation.set(status);
            }

            set_notes.set(data.reason);

            set_native_region.set(data.native_region);
            set_native_latitude.set(data.native_latitude);
            set_native_longitude.set(data.native_longitude);

            if let Some(v) = data.temp_min {
                set_temp_min.set(v.to_string());
            }
            if let Some(v) = data.temp_max {
                set_temp_max.set(v.to_string());
            }
            if let Some(v) = data.humidity_min {
                set_humidity_min.set(v.to_string());
            }
            if let Some(v) = data.humidity_max {
                set_humidity_max.set(v.to_string());
            }

            // Seasonal prefill
            set_rest_start_month.set(data.rest_start_month);
            set_rest_end_month.set(data.rest_end_month);
            set_bloom_start_month.set(data.bloom_start_month);
            set_bloom_end_month.set(data.bloom_end_month);
            set_rest_water_mult.set(data.rest_water_multiplier);
            set_rest_fert_mult.set(data.rest_fertilizer_multiplier);
            set_active_water_mult.set(data.active_water_multiplier);
            set_active_fert_mult.set(data.active_fertilizer_multiplier);
        }
    });

    let on_auto_calculate = move |_ev: leptos::ev::MouseEvent| {
        let size =
            serde_json::from_str::<crate::orchid::PotSize>(&format!("\"{}\"", pot_size.get()))
                .unwrap_or_default();
        let medium =
            serde_json::from_str::<crate::orchid::PotMedium>(&format!("\"{}\"", pot_medium.get()))
                .unwrap_or_default();
        let p_type =
            serde_json::from_str::<crate::orchid::PotType>(&format!("\"{}\"", pot_type.get()))
                .unwrap_or_default();

        let light_req = match light.get().as_str() {
            "Low" => LightRequirement::Low,
            "High" => LightRequirement::High,
            _ => LightRequirement::Medium,
        };

        let par: Option<f64> = par_ppfd.get().parse().ok();

        let days = crate::estimation::calculate_algorithmic_base_days(
            &size,
            &medium,
            &p_type,
            &light_req,
            crate::estimation::VPD_BASELINE, // Use standard home VPD since zone isn't created yet or we might not have a reading
            par,
        );
        set_water_freq.set(days.to_string());
    };

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let light_req = match light.get().as_str() {
            "Low" => LightRequirement::Low,
            "High" => LightRequirement::High,
            _ => LightRequirement::Medium,
        };

        let cons_status = conservation.get();
        let conservation_opt = if cons_status.is_empty() {
            None
        } else {
            Some(cons_status)
        };

        let new_orchid = Orchid {
            id: String::new(),
            name: name.get(),
            species: species.get(),
            water_frequency_days: water_freq.get().parse().unwrap_or(7),
            light_requirement: light_req,
            notes: notes.get(),
            placement: placement.get(),
            light_lux: lux.get(),
            temperature_range: temp.get(),
            conservation_status: conservation_opt,
            native_region: native_region.get(),
            native_latitude: native_latitude.get(),
            native_longitude: native_longitude.get(),
            last_watered_at: None,
            temp_min: temp_min.get().parse().ok(),
            temp_max: temp_max.get().parse().ok(),
            humidity_min: humidity_min.get().parse().ok(),
            humidity_max: humidity_max.get().parse().ok(),
            first_bloom_at: None,
            last_fertilized_at: None,
            fertilize_frequency_days: None,
            fertilizer_type: None,
            last_repotted_at: None,
            pot_medium: if pot_medium.get().is_empty() {
                None
            } else {
                serde_json::from_str(&format!("\"{}\"", pot_medium.get())).ok()
            },
            pot_size: if pot_size.get().is_empty() {
                None
            } else {
                serde_json::from_str(&format!("\"{}\"", pot_size.get())).ok()
            },
            pot_type: if pot_type.get().is_empty() {
                None
            } else {
                serde_json::from_str(&format!("\"{}\"", pot_type.get())).ok()
            },
            rest_start_month: rest_start_month.get(),
            rest_end_month: rest_end_month.get(),
            bloom_start_month: bloom_start_month.get(),
            bloom_end_month: bloom_end_month.get(),
            rest_water_multiplier: rest_water_mult.get(),
            rest_fertilizer_multiplier: rest_fert_mult.get(),
            active_water_multiplier: active_water_mult.get(),
            active_fertilizer_multiplier: active_fert_mult.get(),
            par_ppfd: par_ppfd.get().parse().ok(),
        };

        on_add(new_orchid);
        on_close();

        set_name.set(String::new());
        set_species.set(String::new());
        set_water_freq.set("7".to_string());
        set_light.set("Medium".to_string());
        set_placement.set(String::new());
        set_notes.set(String::new());
        set_lux.set(String::new());
        set_temp.set(String::new());
        set_conservation.set(String::new());
        set_pot_medium.set(String::new());
        set_pot_size.set(String::new());
        set_pot_type.set(String::new());
        set_par_ppfd.set(String::new());
    };

    view! {
        <div class=MODAL_OVERLAY>
            <div class=MODAL_CONTENT>
                <div class=MODAL_HEADER>
                    <h2 class="m-0">"Add New Plant"</h2>
                    <button class=BTN_CLOSE on:click=move |_| on_close()>"Close"</button>
                </div>
                <div>
                    <form on:submit=on_submit>
                        <div class="mb-4">
                            <label>"Name:"</label>
                            <input type="text"
                                on:input=move |ev| set_name.set(event_target_value(&ev))
                                prop:value=name
                                required
                            />
                        </div>
                        <div class="mb-4">
                            <label>"Species:"</label>
                            <input type="text"
                                on:input=move |ev| set_species.set(event_target_value(&ev))
                                prop:value=species
                                required
                            />
                        </div>
                        <div class="mb-4">
                            <label>"Conservation Status (e.g. CITES II):"</label>
                            <input type="text"
                                on:input=move |ev| set_conservation.set(event_target_value(&ev))
                                prop:value=conservation
                            />
                        </div>
                        <div class="flex flex-col gap-4 mb-4 sm:flex-row">
                            <div class="flex-1">
                                <div class="flex justify-between items-center">
                                    <label>"Water Freq (days):"</label>
                                    <button
                                        type="button"
                                        class="transition-colors focus:outline-none text-[10px] text-primary hover:text-primary-light"
                                        on:click=on_auto_calculate
                                        title="Auto-calculate based on pot size, medium, and type"
                                    >
                                        "\u{2728} Auto-Calc"
                                    </button>
                                </div>
                                <input type="number"
                                    on:input=move |ev| set_water_freq.set(event_target_value(&ev))
                                    prop:value=water_freq
                                    required
                                />
                            </div>
                            <div class="flex-1">
                                <label>"Light Req:"</label>
                                <select
                                    on:change=move |ev| set_light.set(event_target_value(&ev))
                                    prop:value=light
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
                                    on:change=move |ev| set_placement.set(event_target_value(&ev))
                                    prop:value=placement
                                >
                                    {zones.iter().map(|zone| {
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
                                    on:input=move |ev| set_lux.set(event_target_value(&ev))
                                    prop:value=lux
                                    placeholder="e.g. 5000"
                                />
                            </div>
                            <div class="flex-1">
                                <label>"PAR (PPFD):"</label>
                                <input type="number" step="1" min="0" max="2500"
                                    on:input=move |ev| set_par_ppfd.set(event_target_value(&ev))
                                    prop:value=par_ppfd
                                    placeholder="\u{00B5}mol/m\u{00B2}/s"
                                />
                            </div>
                        </div>
                         <div class="mb-4">
                            <label>"Temp Range (Optional):"</label>
                            <input type="text"
                                on:input=move |ev| set_temp.set(event_target_value(&ev))
                                prop:value=temp
                                placeholder="e.g. 18-28C"
                            />
                        </div>
                        <div class="flex flex-col gap-4 mb-4 sm:flex-row">
                            <div class="flex-1">
                                <label>"Min Temp (C):"</label>
                                <input type="number" step="0.1"
                                    on:input=move |ev| set_temp_min.set(event_target_value(&ev))
                                    prop:value=temp_min
                                    placeholder="e.g. 18"
                                />
                            </div>
                            <div class="flex-1">
                                <label>"Max Temp (C):"</label>
                                <input type="number" step="0.1"
                                    on:input=move |ev| set_temp_max.set(event_target_value(&ev))
                                    prop:value=temp_max
                                    placeholder="e.g. 28"
                                />
                            </div>
                        </div>
                        <div class="flex flex-col gap-4 mb-4 sm:flex-row">
                            <div class="flex-1">
                                <label>"Min Humidity (%):"</label>
                                <input type="number" step="0.1"
                                    on:input=move |ev| set_humidity_min.set(event_target_value(&ev))
                                    prop:value=humidity_min
                                    placeholder="e.g. 50"
                                />
                            </div>
                            <div class="flex-1">
                                <label>"Max Humidity (%):"</label>
                                <input type="number" step="0.1"
                                    on:input=move |ev| set_humidity_max.set(event_target_value(&ev))
                                    prop:value=humidity_max
                                    placeholder="e.g. 80"
                                />
                            </div>
                        </div>

                        <PotMediumSection
                            pot_medium=pot_medium set_pot_medium=set_pot_medium
                            pot_size=pot_size set_pot_size=set_pot_size
                            pot_type=pot_type set_pot_type=set_pot_type
                        />

                        <div class="mb-4">
                            <label>"Notes:"</label>
                            <textarea
                                on:input=move |ev| set_notes.set(event_target_value(&ev))
                                prop:value=notes
                                rows="3"
                            ></textarea>
                        </div>
                        <button type="submit" class=format!("{} w-full mt-2", BTN_PRIMARY)>"Add Orchid"</button>
                    </form>
                </div>
            </div>
        </div>
    }.into_any()
}

#[component]
fn PotMediumSection(
    pot_medium: ReadSignal<String>, set_pot_medium: WriteSignal<String>,
    pot_size: ReadSignal<String>, set_pot_size: WriteSignal<String>,
    pot_type: ReadSignal<String>, set_pot_type: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <div class="pt-4 mt-4 mb-4 border-t border-stone-200 dark:border-stone-700">
            <h4 class="mt-0 mb-3 text-xs font-semibold tracking-widest uppercase text-stone-500 dark:text-stone-400">"Pot & Medium Setup"</h4>
            <p class="mb-4 text-xs text-stone-500">"Tracking your pot type and medium helps adjust watering schedules automatically based on evaporation rates."</p>
            <div class="flex flex-col gap-4 sm:flex-row">
                {move || (pot_type.get() != "Mounted").then(|| view! {
                    <div class="flex-1 animate-fade-in">
                        <label>"Pot Medium:"</label>
                        <select
                            on:change=move |ev| set_pot_medium.set(event_target_value(&ev))
                            prop:value=pot_medium
                        >
                            <option value="">"Unknown / Unset"</option>
                            <option value="Bark">"Bark"</option>
                            <option value="Sphagnum Moss">"Sphagnum Moss"</option>
                            <option value="LECA">"LECA (Semi-Hydro)"</option>
                            <option value="Inorganic">"Inorganic (Lava/Pumice)"</option>
                        </select>
                    </div>
                })}
                <div class="flex-1">
                    <label>"Pot Type (Airflow):"</label>
                    <select
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            set_pot_type.set(val.clone());
                            if val == "Mounted" {
                                set_pot_medium.set(String::new());
                                set_pot_size.set(String::new());
                            }
                        }
                        prop:value=pot_type
                    >
                        <option value="">"Unknown / Unset"</option>
                        <option value="Solid">"Solid (Plastic/Glazed)"</option>
                        <option value="Slotted">"Slotted (Net Pot)"</option>
                        <option value="Clay">"Terra Cotta (Clay)"</option>
                        <option value="Mounted">"Mounted (Slab)"</option>
                    </select>
                </div>
                {move || (pot_type.get() != "Mounted").then(|| view! {
                    <div class="flex-1 animate-fade-in">
                        <label>"Pot Size:"</label>
                        <select
                            on:change=move |ev| set_pot_size.set(event_target_value(&ev))
                            prop:value=pot_size
                        >
                            <option value="">"Unknown / Unset"</option>
                            <option value="Small">"Small (2-3\")"</option>
                            <option value="Medium">"Medium (4-5\")"</option>
                            <option value="Large">"Large (6\"+)"</option>
                        </select>
                    </div>
                })}
            </div>
        </div>
    }.into_any()
}
