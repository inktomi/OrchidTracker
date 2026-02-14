use leptos::*;
use gloo_storage::{LocalStorage, Storage};
use crate::orchid::{Orchid, LightRequirement, Placement};

#[component]
pub fn App() -> impl IntoView {
    // State: List of Orchids
    let (orchids, set_orchids) = create_signal(
        LocalStorage::get("orchids").unwrap_or_else(|_| {
             let initial_data = include_str!("data/orchids.json");
             serde_json::from_str(initial_data).unwrap_or_else(|_| Vec::<Orchid>::new())
        })
    );

    // Effect: Persist orchids to LocalStorage whenever they change
    create_effect(move |_| {
        let current_orchids = orchids.get();
        if let Err(e) = LocalStorage::set("orchids", &current_orchids) {
            log::error!("Failed to save to local storage: {:?}", e);
        }
    });

    // Add Orchid Logic
    let add_orchid = move |new_orchid: Orchid| {
        set_orchids.update(|orchids| orchids.push(new_orchid));
    };

    // Delete Orchid Logic
    let delete_orchid = move |id: u64| {
        set_orchids.update(|orchids| {
            orchids.retain(|o| o.id != id);
        });
    };

    view! {
        <header>
            <h1>"Orchid Tracker"</h1>
        </header>
        <main>
            <AddOrchidForm on_add=add_orchid />
            <div class="orchid-grid">
                <For
                    each=move || orchids.get()
                    key=|orchid| orchid.id
                    children=move |orchid| {
                        let orchid_clone = orchid.clone();
                        view! {
                            <OrchidCard orchid=orchid_clone on_delete=delete_orchid />
                        }
                    }
                />
            </div>
        </main>
    }
}

#[component]
fn OrchidCard<F>(orchid: Orchid, on_delete: F) -> impl IntoView
where
    F: Fn(u64) + 'static + Copy,
{
    let orchid_id = orchid.id;
    let suggested = orchid.suggested_placement();
    let is_misplaced = orchid.placement != suggested;
    let suggestion_msg = if is_misplaced {
        format!("(Should be {})", suggested)
    } else {
        " (Optimal)".to_string()
    };
    
    let suggestion_style = if is_misplaced { "color: red; font-weight: bold;" } else { "color: green;" };

    view! {
        <div class="orchid-card">
            <h3>{orchid.name}</h3>
            <p><strong>"Species: "</strong> {orchid.species}</p>
            <p><strong>"Watering: "</strong> "Every " {orchid.water_frequency_days} " days"</p>
            <p><strong>"Light Req: "</strong> {orchid.light_requirement.to_string()} " (" {orchid.light_lux} " Lux)"</p>
            <p><strong>"Temp Range: "</strong> {orchid.temperature_range}</p>
            <p><strong>"Placement: "</strong> {orchid.placement.to_string()}</p>
            <p style=suggestion_style><strong>"Suggestion: "</strong> {suggestion_msg}</p>
            <p><strong>"Notes: "</strong> {orchid.notes}</p>
            <button class="delete-btn" on:click=move |_| on_delete(orchid_id)>"Delete"</button>
        </div>
    }
}

#[component]
fn AddOrchidForm<F>(on_add: F) -> impl IntoView
where
    F: Fn(Orchid) + 'static,
{
    let (name, set_name) = create_signal("".to_string());
    let (species, set_species) = create_signal("".to_string());
    let (water_freq, set_water_freq) = create_signal("7".to_string());
    let (light, set_light) = create_signal("Medium".to_string());
    let (placement, set_placement) = create_signal("Medium".to_string());
    let (notes, set_notes) = create_signal("".to_string());
    let (lux, set_lux) = create_signal("".to_string());
    let (temp, set_temp) = create_signal("".to_string());

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        
        let id = js_sys::Date::now() as u64;
        let light_req = match light.get().as_str() {
            "Low" => LightRequirement::Low,
            "High" => LightRequirement::High,
            _ => LightRequirement::Medium,
        };
        let place = match placement.get().as_str() {
             "Low" => Placement::Low,
             "High" => Placement::High,
             _ => Placement::Medium,
        };
        
        let new_orchid = Orchid::new(
            id,
            name.get(),
            species.get(),
            water_freq.get().parse().unwrap_or(7),
            light_req,
            notes.get(),
            place,
            lux.get(),
            temp.get(),
        );
        
        on_add(new_orchid);
        
        // Reset form
        set_name.set("".to_string());
        set_species.set("".to_string());
        set_water_freq.set("7".to_string());
        set_light.set("Medium".to_string());
        set_placement.set("Medium".to_string());
        set_notes.set("".to_string());
        set_lux.set("".to_string());
        set_temp.set("".to_string());
    };

    view! {
        <div class="form-container">
            <h2>"Add New Orchid"</h2>
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
                    <label>"Water Frequency (days):"</label>
                    <input type="number"
                        on:input=move |ev| set_water_freq.set(event_target_value(&ev))
                        prop:value=water_freq
                        required
                    />
                </div>
                <div class="form-group">
                    <label>"Light Requirement:"</label>
                    <select
                        on:change=move |ev| set_light.set(event_target_value(&ev))
                        prop:value=light
                    >
                        <option value="Low">"Low Light"</option>
                        <option value="Medium">"Medium Light"</option>
                        <option value="High">"High Light"</option>
                    </select>
                </div>
                <div class="form-group">
                    <label>"Current Placement:"</label>
                    <select
                        on:change=move |ev| set_placement.set(event_target_value(&ev))
                        prop:value=placement
                    >
                        <option value="Low">"Low Light Area"</option>
                        <option value="Medium">"Medium Light Area"</option>
                        <option value="High">"High Light Area"</option>
                    </select>
                </div>
                 <div class="form-group">
                    <label>"Light (Lux) (Optional):"</label>
                    <input type="text"
                        on:input=move |ev| set_lux.set(event_target_value(&ev))
                        prop:value=lux
                    />
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
                    ></textarea>
                </div>
                <button type="submit">"Add Orchid"</button>
            </form>
        </div>
    }
}
