use leptos::*;
use gloo_storage::{LocalStorage, Storage};
use crate::orchid::{Orchid, LightRequirement, Placement};
use crate::components::orchid_detail::OrchidDetail;
use crate::components::settings::SettingsModal;
use crate::components::scanner::{ScannerModal, AnalysisResult};
use crate::github::sync_orchids_to_github;
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
    
    // State: Show Settings Modal
    let (show_settings, set_show_settings) = create_signal(false);
    
    // State: Show Scanner Modal
    let (show_scanner, set_show_scanner) = create_signal(false);
    
    // State: Show Add Orchid Modal
    let (show_add_modal, set_show_add_modal) = create_signal(false);
    
    // State: Pre-fill data for Add Form
    let (prefill_data, set_prefill_data) = create_signal::<Option<AnalysisResult>>(None);

    // State: Temperature Unit
    let (temp_unit, set_temp_unit) = create_signal("C".to_string());
    if let Ok(u) = LocalStorage::get("temp_unit") {
        set_temp_unit.set(u);
    }

    // State: Sync Status
    let (sync_status, set_sync_status) = create_signal("".to_string());

    // Effect: Persist orchids to LocalStorage whenever they change
    create_effect(move |_| {
        let current_orchids = orchids.get();
        if let Err(e) = LocalStorage::set("orchids", &current_orchids) {
            log::error!("Failed to save to local storage: {:?}", e);
        }
    });
    
    // Function to trigger GitHub Sync
    let trigger_sync = move |current_orchids: Vec<Orchid>| {
        set_sync_status.set("Syncing...".to_string());
        spawn_local(async move {
            match sync_orchids_to_github(current_orchids).await {
                Ok(_) => set_sync_status.set("Synced!".to_string()),
                Err(e) => {
                    log::error!("Sync failed: {}", e);
                    set_sync_status.set("Sync Failed".to_string());
                }
            }
            // Clear status after 3s
            gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await;
            set_sync_status.set("".to_string());
        });
    };

    // Add Orchid Logic
    let add_orchid = move |new_orchid: Orchid| {
        set_orchids.update(|orchids| orchids.push(new_orchid.clone()));
        // Trigger Sync
        trigger_sync(orchids.get());
    };

    // Update Orchid Logic (for notes/history)
    let update_orchid = move |updated_orchid: Orchid| {
        set_orchids.update(|orchids| {
            if let Some(pos) = orchids.iter().position(|o| o.id == updated_orchid.id) {
                orchids[pos] = updated_orchid;
            }
        });
        // Trigger Sync
        trigger_sync(orchids.get());
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
        // Trigger Sync
        trigger_sync(orchids.get());
    };
    
    // Handle Scan Result
    let handle_scan_result = move |result: AnalysisResult| {
        set_prefill_data.set(Some(result));
        set_show_scanner.set(false);
        set_show_add_modal.set(true);
    };

    view! {
        <header>
            <div class="header-top">
                <h1>"Orchid Tracker"</h1>
                <div class="header-controls">
                    <span class="sync-status">{sync_status}</span>
                    <button class="action-btn" on:click=move |_| trigger_sync(orchids.get())>"🔄 Sync"</button>
                    <button class="action-btn" on:click=move |_| set_show_add_modal.set(true)>"➕ Add"</button>
                    <button class="action-btn" on:click=move |_| set_show_scanner.set(true)>"📷 Scan"</button>
                    <button class="settings-btn" on:click=move |_| set_show_settings.set(true)>"⚙️ Settings"</button>
                </div>
            </div>
            
            <ClimateDashboard data=climate_data.clone() unit=temp_unit />

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
                    "Placement View"
                </button>
            </div>
        </header>
        <main>
            {move || if show_add_modal.get() {
                view! {
                    <AddOrchidForm 
                        on_add=add_orchid 
                        on_close=move || set_show_add_modal.set(false)
                        prefill_data=prefill_data 
                    />
                }.into_view()
            } else {
                view! {}.into_view()
            }}
            
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
                        on_update=update_orchid
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
            
            // Settings Modal
            {move || if show_settings.get() {
                view! {
                    <SettingsModal on_close=move || {
                        set_show_settings.set(false);
                        // Refresh prefs
                        if let Ok(u) = LocalStorage::get("temp_unit") {
                            set_temp_unit.set(u);
                        }
                    } />
                }.into_view()
            } else {
                view! {}.into_view()
            }}
            
            // Scanner Modal
            {move || if show_scanner.get() {
                let summary = if !climate_data.is_empty() {
                    let d = &climate_data[0];
                    format!("Temp: {}C, Humidity: {}%, VPD: {}kPa", d.temperature, d.humidity, d.vpd)
                } else {
                    "Unknown climate".to_string()
                };
                
                view! {
                    <ScannerModal 
                        on_close=move || set_show_scanner.set(false)
                        on_add_to_collection=handle_scan_result
                        existing_orchids=orchids.get()
                        climate_summary=summary
                    />
                }.into_view()
            } else {
                 view! {}.into_view()
            }}
        </main>
    }
}

#[component]
fn ClimateDashboard(data: Vec<ClimateData>, unit: ReadSignal<String>) -> impl IntoView {
    if data.is_empty() {
        view! { <div class="climate-dashboard empty">"No climate data available (Configure AC Infinity Action)"</div> }.into_view()
    } else {
        view! {
            <div class="climate-dashboard-container">
                {move || {
                    let u = unit.get();
                    data.iter().map(|dev| {
                        let (temp_val, temp_unit_str) = if u == "F" {
                            let f = (dev.temperature * 9.0 / 5.0) + 32.0;
                            (format!("{:.1}", f), "°F")
                        } else {
                            (format!("{:.1}", dev.temperature), "°C")
                        };

                        view! {
                            <div class="climate-dashboard">
                                <h3>{dev.name.clone()}</h3>
                                <div class="climate-stat">
                                    <span class="label">"Temperature"</span>
                                    <span class="value">{temp_val} " " {temp_unit_str}</span>
                                </div>
                                <div class="climate-stat">
                                    <span class="label">"Humidity"</span>
                                    <span class="value">{dev.humidity} "%"</span>
                                </div>
                                <div class="climate-stat">
                                    <span class="label">"VPD"</span>
                                    <span class="value">{dev.vpd} " kPa"</span>
                                </div>
                                <div class="climate-footer">
                                    "Last Updated: " {dev.updated.clone()}
                                </div>
                            </div>
                        }
                    }).collect_view()
                }}
            </div>
        }.into_view()
    }
}

#[component]
fn OrchidCabinetTable<F, S, U>(orchids: Vec<Orchid>, on_delete: F, on_select: S, on_update: U) -> impl IntoView
where
    F: Fn(u64) + 'static + Copy,
    S: Fn(Orchid) + 'static + Copy,
    U: Fn(Orchid) + 'static + Copy,
{
    // Filter orchids by placement
    let high_orchids: Vec<Orchid> = orchids.iter().filter(|o| o.placement == Placement::High).cloned().collect();
    let medium_orchids: Vec<Orchid> = orchids.iter().filter(|o| o.placement == Placement::Medium).cloned().collect();
    let low_orchids: Vec<Orchid> = orchids.iter().filter(|o| o.placement == Placement::Low).cloned().collect();
    let patio_orchids: Vec<Orchid> = orchids.iter().filter(|o| o.placement == Placement::Patio).cloned().collect();
    let outdoor_orchids: Vec<Orchid> = orchids.iter().filter(|o| o.placement == Placement::OutdoorRack).cloned().collect();

    // Helper to handle drop
    let handle_drop = move |ev: leptos::ev::DragEvent, new_placement: Placement| {
        ev.prevent_default();
        if let Some(data) = ev.data_transfer() {
            if let Ok(id_str) = data.get_data("text/plain") {
                if let Ok(id) = id_str.parse::<u64>() {
                     if let Some(mut orchid) = orchids.iter().find(|o| o.id == id).cloned() {
                         if orchid.placement != new_placement {
                             orchid.placement = new_placement;
                             on_update(orchid);
                         }
                     }
                }
            }
        }
    };

    view! {
        <div class="cabinet-view">
            <h2>"Orchidarium Layout"</h2>
            
            <div 
                class="cabinet-section high-section"
                on:dragover=move |ev| ev.prevent_default()
                on:drop={
                    let handle_drop = handle_drop.clone();
                    move |ev| handle_drop(ev, Placement::High)
                }
            >
                <h3>"Top Shelf (High Light - Near Lights)"</h3>
                <OrchidTableSection orchids=high_orchids on_delete=on_delete on_select=on_select />
            </div>

            <div 
                class="cabinet-section medium-section"
                on:dragover=move |ev| ev.prevent_default()
                on:drop={
                    let handle_drop = handle_drop.clone();
                    move |ev| handle_drop(ev, Placement::Medium)
                }
            >
                <h3>"Middle Shelf (Medium Light)"</h3>
                <OrchidTableSection orchids=medium_orchids on_delete=on_delete on_select=on_select />
            </div>

            <div 
                class="cabinet-section low-section"
                on:dragover=move |ev| ev.prevent_default()
                on:drop={
                    let handle_drop = handle_drop.clone();
                    move |ev| handle_drop(ev, Placement::Low)
                }
            >
                <h3>"Bottom Shelf (Low Light - Floor)"</h3>
                <OrchidTableSection orchids=low_orchids on_delete=on_delete on_select=on_select />
            </div>
            
            <h2>"Outdoors"</h2>
            
            <div 
                class="cabinet-section outdoor-section"
                on:dragover=move |ev| ev.prevent_default()
                on:drop={
                    let handle_drop = handle_drop.clone();
                    move |ev| handle_drop(ev, Placement::OutdoorRack)
                }
            >
                <h3>"Outdoor Rack (High Sun)"</h3>
                <OrchidTableSection orchids=outdoor_orchids on_delete=on_delete on_select=on_select />
            </div>

            <div 
                class="cabinet-section patio-section"
                on:dragover=move |ev| ev.prevent_default()
                on:drop={
                    let handle_drop = handle_drop.clone();
                    move |ev| handle_drop(ev, Placement::Patio)
                }
            >
                <h3>"Patio (Morning Sun / Afternoon Shade)"</h3>
                <OrchidTableSection orchids=patio_orchids on_delete=on_delete on_select=on_select />
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
                            let is_misplaced = !orchid.placement.is_compatible_with(&orchid.light_requirement);
                            let status_class = if is_misplaced { "status-warning" } else { "status-ok" };
                            let status_text = if is_misplaced { "Move Needed" } else { "OK" };
                            
                            view! {
                                <tr 
                                    class="clickable-row" 
                                    draggable="true"
                                    on:click=move |_| on_select(orchid_clone.clone())
                                    on:dragstart=move |ev| {
                                        if let Some(data) = ev.data_transfer() {
                                            let _ = data.set_data("text/plain", &orchid_id.to_string());
                                        }
                                    }
                                >
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
    let is_misplaced = !orchid.placement.is_compatible_with(&orchid.light_requirement);
    let suggestion_msg = if is_misplaced {
        format!("(Needs {})", orchid.light_requirement)
    } else {
        " (Optimal)".to_string()
    };
    
    let suggestion_style = if is_misplaced { "color: red; font-weight: bold;" } else { "color: green;" };

    view! {
        <div class="orchid-card">
            <div class="card-content" on:click=move |_| on_select(orchid_clone.clone())>
                <h3>{orchid.name}</h3>
                <p><strong>"Species: "</strong> {orchid.species}</p>
                
                {if let Some(status) = orchid.conservation_status {
                    view! { <p class="conservation-status"><strong>"Status: "</strong> {status}</p> }.into_view()
                } else {
                    view! {}.into_view()
                }}
                
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
fn AddOrchidForm<F, C>(on_add: F, on_close: C, prefill_data: ReadSignal<Option<AnalysisResult>>) -> impl IntoView
where
    F: Fn(Orchid) + 'static,
    C: Fn() + 'static + Copy,
{
    let (name, set_name) = create_signal("".to_string());
    let (species, set_species) = create_signal("".to_string());
    let (water_freq, set_water_freq) = create_signal("7".to_string());
    let (light, set_light) = create_signal("Medium".to_string());
    let (placement, set_placement) = create_signal("Medium".to_string());
    let (notes, set_notes) = create_signal("".to_string());
    let (lux, set_lux) = create_signal("".to_string());
    let (temp, set_temp) = create_signal("".to_string());
    let (conservation, set_conservation) = create_signal("".to_string());

    // Effect to pre-fill form when scanner result comes in
    create_effect(move |_| {
        if let Some(data) = prefill_data.get() {
            set_name.set(data.species_name.clone());
            set_species.set(data.species_name);
            set_water_freq.set(data.water_freq.to_string());
            
            // Map AI light req to form values
            let light_val = match data.light_req.to_lowercase().as_str() {
                "low" | "low light" => "Low",
                "high" | "high light" => "High",
                _ => "Medium",
            };
            set_light.set(light_val.to_string());
            
            // Map AI placement to form values
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
        
        let id = js_sys::Date::now() as u64;
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
            conservation_opt,
        );
        
        on_add(new_orchid);
        on_close();
        
        // Reset form
        set_name.set("".to_string());
        set_species.set("".to_string());
        set_water_freq.set("7".to_string());
        set_light.set("Medium".to_string());
        set_placement.set("Medium".to_string());
        set_notes.set("".to_string());
        set_lux.set("".to_string());
        set_temp.set("".to_string());
        set_conservation.set("".to_string());
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
