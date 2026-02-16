use leptos::prelude::*;
use crate::orchid::{Orchid, LightRequirement, Placement, generate_id};
use crate::components::scanner::AnalysisResult;

#[component]
pub fn AddOrchidForm(
    on_add: impl Fn(Orchid) + 'static + Send + Sync,
    on_close: impl Fn() + 'static + Copy + Send + Sync,
    prefill_data: ReadSignal<Option<AnalysisResult>>,
) -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (species, set_species) = signal(String::new());
    let (water_freq, set_water_freq) = signal("7".to_string());
    let (light, set_light) = signal("Medium".to_string());
    let (placement, set_placement) = signal("Medium".to_string());
    let (notes, set_notes) = signal(String::new());
    let (lux, set_lux) = signal(String::new());
    let (temp, set_temp) = signal(String::new());
    let (conservation, set_conservation) = signal(String::new());

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

             let place_val = match data.placement_suggestion.to_lowercase().as_str() {
                "low" | "low light" => "Low",
                "high" | "high light" => "High",
                "patio" => "Patio",
                "outdoorrack" | "outdoor rack" => "OutdoorRack",
                _ => "Medium",
            };
            set_placement.set(place_val.to_string());

            set_temp.set(data.temp_range);

            if let Some(status) = data.conservation_status {
                set_conservation.set(status);
            }

            let note_text = format!("AI Analysis: {}\nReason: {}", data.fit_category, data.reason);
            set_notes.set(note_text);
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let id = generate_id();
        let light_req = match light.get().as_str() {
            "Low" => LightRequirement::Low,
            "High" => LightRequirement::High,
            _ => LightRequirement::Medium,
        };
        let place = match placement.get().as_str() {
             "Low" => Placement::Low,
             "High" => Placement::High,
             "Patio" => Placement::Patio,
             "OutdoorRack" => Placement::OutdoorRack,
             _ => Placement::Medium,
        };

        let cons_status = conservation.get();
        let conservation_opt = if cons_status.is_empty() { None } else { Some(cons_status) };

        let new_orchid = Orchid {
            id,
            name: name.get(),
            species: species.get(),
            water_frequency_days: water_freq.get().parse().unwrap_or(7),
            light_requirement: light_req,
            notes: notes.get(),
            placement: place,
            light_lux: lux.get(),
            temperature_range: temp.get(),
            conservation_status: conservation_opt,
            history: Vec::new(),
        };

        on_add(new_orchid);
        on_close();

        set_name.set(String::new());
        set_species.set(String::new());
        set_water_freq.set("7".to_string());
        set_light.set("Medium".to_string());
        set_placement.set("Medium".to_string());
        set_notes.set(String::new());
        set_lux.set(String::new());
        set_temp.set(String::new());
        set_conservation.set(String::new());
    };

    view! {
        <div class="modal-overlay">
            <div class="modal-content form-modal">
                <div class="modal-header">
                    <h2>"Add New Orchid"</h2>
                    <button class="close-btn" on:click=move |_| on_close()>"X"</button>
                </div>
                <div class="modal-body">
                    <form on:submit=on_submit>
                        <div class="form-group">
                            <label>"Name:"</label>
                            <input type="text"
                                on:input=move |ev| set_name.set(event_target_value(&ev))
                                prop:value=name
                                required
                            />
                        </div>
                        <div class="form-group">
                            <label>"Species:"</label>
                            <input type="text"
                                on:input=move |ev| set_species.set(event_target_value(&ev))
                                prop:value=species
                                required
                            />
                        </div>
                        <div class="form-group">
                            <label>"Conservation Status (e.g. CITES II):"</label>
                            <input type="text"
                                on:input=move |ev| set_conservation.set(event_target_value(&ev))
                                prop:value=conservation
                            />
                        </div>
                        <div class="form-row">
                            <div class="form-group half-width">
                                <label>"Water Freq (days):"</label>
                                <input type="number"
                                    on:input=move |ev| set_water_freq.set(event_target_value(&ev))
                                    prop:value=water_freq
                                    required
                                />
                            </div>
                            <div class="form-group half-width">
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
                        <div class="form-row">
                             <div class="form-group half-width">
                                <label>"Placement:"</label>
                                <select
                                    on:change=move |ev| set_placement.set(event_target_value(&ev))
                                    prop:value=placement
                                >
                                    <option value="Low">"Low Light Area"</option>
                                    <option value="Medium">"Medium Light Area"</option>
                                    <option value="High">"High Light Area"</option>
                                    <option value="Patio">"Patio (Outdoors)"</option>
                                    <option value="OutdoorRack">"Outdoor Rack"</option>
                                </select>
                            </div>
                             <div class="form-group half-width">
                                <label>"Light (Lux):"</label>
                                <input type="text"
                                    on:input=move |ev| set_lux.set(event_target_value(&ev))
                                    prop:value=lux
                                />
                            </div>
                        </div>
                         <div class="form-group">
                            <label>"Temp Range (Optional):"</label>
                            <input type="text"
                                on:input=move |ev| set_temp.set(event_target_value(&ev))
                                prop:value=temp
                            />
                        </div>
                        <div class="form-group">
                            <label>"Notes:"</label>
                            <textarea
                                on:input=move |ev| set_notes.set(event_target_value(&ev))
                                prop:value=notes
                                rows="3"
                            ></textarea>
                        </div>
                        <button type="submit" class="submit-btn">"Add Orchid"</button>
                    </form>
                </div>
            </div>
        </div>
    }
}
