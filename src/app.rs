use leptos::*;
use gloo_storage::{LocalStorage, Storage};
use crate::orchid::{Orchid, LightRequirement, Placement};
use crate::components::orchid_detail::OrchidDetail;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq)]
enum ViewMode {
    Grid,
    Table,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClimateData {
    pub name: String,
    pub type_str: Option<String>,
    pub temperature: f64,
    pub humidity: f64,
    pub vpd: f64,
    pub updated: String,
}

#[component]
pub fn App() -> impl IntoView {
    // State: List of Orchids
    let (orchids, set_orchids) = create_signal(
        LocalStorage::get("orchids").unwrap_or_else(|_| {
             let initial_data = include_str!("data/orchids.json");
             serde_json::from_str(initial_data).unwrap_or_else(|_| Vec::<Orchid>::new())
        })
    );

    // Load Climate Data (Snapshot from GitHub Action)
    let climate_data: Vec<ClimateData> = serde_json::from_str(include_str!("data/climate.json"))
        .unwrap_or_else(|_| Vec::new());

    // State: View Mode
    let (view_mode, set_view_mode) = create_signal(ViewMode::Grid);
    
    // State: Selected Orchid for Detail View
    let (selected_orchid, set_selected_orchid) = create_signal::<Option<Orchid>>(None);

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

    // Update Orchid Logic (for notes/history)
    let update_orchid = move |updated_orchid: Orchid| {
        set_orchids.update(|orchids| {
            if let Some(pos) = orchids.iter().position(|o| o.id == updated_orchid.id) {
                orchids[pos] = updated_orchid;
            }
        });
    };

    // Delete Orchid Logic
    let delete_orchid = move |id: u64| {
        set_orchids.update(|orchids| {
            orchids.retain(|o| o.id != id);
        });
        // If the deleted orchid was selected, close the modal
        if let Some(selected) = selected_orchid.get() {
            if selected.id == id {
                set_selected_orchid.set(None);
            }
        }
    };

    view! {
        <header>
            <h1>"Orchid Tracker"</h1>
            
            <ClimateDashboard data=climate_data />

            <div class="view-toggle">
                <button 
                    class=move || if view_mode.get() == ViewMode::Grid { "active" } else { "" }
                    on:click=move |_| set_view_mode.set(ViewMode::Grid)
                >
                    "Grid View"
                </button>
                <button 
                    class=move || if view_mode.get() == ViewMode::Table { "active" } else { "" }
                    on:click=move |_| set_view_mode.set(ViewMode::Table)
                >
                    "Cabinet Table View"
                </button>
            </div>
        </header>
        <main>
            <AddOrchidForm on_add=add_orchid />
            
            {move || match view_mode.get() {
                ViewMode::Grid => view! {
                    <div class="orchid-grid">
                        <For
                            each=move || orchids.get()
                            key=|orchid| orchid.id
                            children=move |orchid| {
                                let orchid_clone = orchid.clone();
                                view! {
                                    <OrchidCard 
                                        orchid=orchid_clone 
                                        on_delete=delete_orchid 
                                        on_select=move |o| set_selected_orchid.set(Some(o))
                                    />
                                }
                            }
                        />
                    </div>
                }.into_view(),
                ViewMode::Table => view! {
                    <OrchidCabinetTable 
                        orchids=orchids.get() 
                        on_delete=delete_orchid 
                        on_select=move |o| set_selected_orchid.set(Some(o))
                    />
                }.into_view()
            }}

            // Detail Modal
            {move || if let Some(orchid) = selected_orchid.get() {
                view! {
                    <OrchidDetail 
                        orchid=orchid 
                        on_close=move || set_selected_orchid.set(None)
                        on_update=update_orchid
                    />
                }.into_view()
            } else {
                view! {}.into_view()
            }}
        </main>
    }
}

#[component]
fn ClimateDashboard(data: Vec<ClimateData>) -> impl IntoView {
    if data.is_empty() {
        view! { <div class="climate-dashboard empty">"No climate data available (Configure AC Infinity Action)"</div> }.into_view()
    } else {
        // Just show the first device (Controller 69 Pro usually)
        let main_dev = &data[0];
        
        view! {
            <div class="climate-dashboard">
                <div class="climate-stat">
                    <span class="label">"Temperature"</span>
                    <span class="value">{main_dev.temperature} "°C"</span>
                </div>
                <div class="climate-stat">
                    <span class="label">"Humidity"</span>
                    <span class="value">{main_dev.humidity} "%"</span>
                </div>
                <div class="climate-stat">
                    <span class="label">"VPD"</span>
                    <span class="value">{main_dev.vpd} " kPa"</span>
                </div>
                <div class="climate-footer">
                    "Last Updated: " {main_dev.updated.clone()}
                </div>
            </div>
        }.into_view()
    }
}

#[component]
fn OrchidCabinetTable<F, S>(orchids: Vec<Orchid>, on_delete: F, on_select: S) -> impl IntoView
where
    F: Fn(u64) + 'static + Copy,
    S: Fn(Orchid) + 'static + Copy,
{
    // Filter orchids by placement
    let high_orchids: Vec<Orchid> = orchids.iter().filter(|o| o.placement == Placement::High).cloned().collect();
    let medium_orchids: Vec<Orchid> = orchids.iter().filter(|o| o.placement == Placement::Medium).cloned().collect();
    let low_orchids: Vec<Orchid> = orchids.iter().filter(|o| o.placement == Placement::Low).cloned().collect();

    view! {
        <div class="cabinet-view">
            <h2>"Orchidarium Layout (6ft Tall)"</h2>
            
            <div class="cabinet-section high-section">
                <h3>"Top Shelf (High Light - Near Lights)"</h3>
                <OrchidTableSection orchids=high_orchids on_delete=on_delete on_select=on_select />
            </div>

            <div class="cabinet-section medium-section">
                <h3>"Middle Shelf (Medium Light)"</h3>
                <OrchidTableSection orchids=medium_orchids on_delete=on_delete on_select=on_select />
            </div>

            <div class="cabinet-section low-section">
                <h3>"Bottom Shelf (Low Light - Floor)"</h3>
                <OrchidTableSection orchids=low_orchids on_delete=on_delete on_select=on_select />
            </div>
        </div>
    }
}

#[component]
fn OrchidTableSection<F, S>(orchids: Vec<Orchid>, on_delete: F, on_select: S) -> impl IntoView
where
    F: Fn(u64) + 'static + Copy,
    S: Fn(Orchid) + 'static + Copy,
{
    if orchids.is_empty() {
        view! { <p class="empty-shelf">"No orchids on this shelf."</p> }.into_view()
    } else {
        view! {
            <table class="orchid-table">
                <thead>
                    <tr>
                        <th>"Name"</th>
                        <th>"Species"</th>
                        <th>"Watering"</th>
                        <th>"Light Req"</th>
                        <th>"Temp Range"</th>
                        <th>"Status"</th>
                        <th>"Actions"</th>
                    </tr>
                </thead>
                <tbody>
                    <For
                        each=move || orchids.clone()
                        key=|orchid| orchid.id
                        children=move |orchid| {
                            let orchid_id = orchid.id;
                            let orchid_clone = orchid.clone();
                            let suggested = orchid.suggested_placement();
                            let is_misplaced = orchid.placement != suggested;
                            let status_class = if is_misplaced { "status-warning" } else { "status-ok" };
                            let status_text = if is_misplaced { "Move Needed" } else { "OK" };
                            
                            view! {
                                <tr class="clickable-row" on:click=move |_| on_select(orchid_clone.clone())>
                                    <td>{orchid.name}</td>
                                    <td>{orchid.species}</td>
                                    <td>"Every " {orchid.water_frequency_days} " days"</td>
                                    <td>{orchid.light_requirement.to_string()}</td>
                                    <td>{orchid.temperature_range}</td>
                                    <td class=status_class>{status_text}</td>
                                    <td>
                                        <button class="delete-btn-small" on:click=move |ev| {
                                            ev.stop_propagation();
                                            on_delete(orchid_id);
                                        }>"X"</button>
                                    </td>
                                </tr>
                            }
                        }
                    />
                </tbody>
            </table>
        }.into_view()
    }
}

#[component]
fn OrchidCard<F, S>(orchid: Orchid, on_delete: F, on_select: S) -> impl IntoView
where
    F: Fn(u64) + 'static + Copy,
    S: Fn(Orchid) + 'static + Copy,
{
    let orchid_id = orchid.id;
    let orchid_clone = orchid.clone();
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
            <div class="card-content" on:click=move |_| on_select(orchid_clone.clone())>
                <h3>{orchid.name}</h3>
                <p><strong>"Species: "</strong> {orchid.species}</p>
                <p><strong>"Watering: "</strong> "Every " {orchid.water_frequency_days} " days"</p>
                <p><strong>"Light Req: "</strong> {orchid.light_requirement.to_string()} " (" {orchid.light_lux} " Lux)"</p>
                <p><strong>"Temp Range: "</strong> {orchid.temperature_range}</p>
                <p><strong>"Placement: "</strong> {orchid.placement.to_string()}</p>
                <p style=suggestion_style><strong>"Suggestion: "</strong> {suggestion_msg}</p>
                <p><strong>"Notes: "</strong> {orchid.notes}</p>
            </div>
            <button class="delete-btn" on:click=move |ev| {
                ev.stop_propagation();
                on_delete(orchid_id);
            }>"Delete"</button>
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
