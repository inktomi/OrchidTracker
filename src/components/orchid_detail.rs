use leptos::prelude::*;
use leptos::task::spawn_local;
use gloo_file::File;
use gloo_storage::{LocalStorage, Storage};
use web_sys::{HtmlInputElement, Url};
use crate::orchid::{Orchid, LightRequirement, Placement};
use crate::db::{save_image_blob, get_image_blob};
use chrono::Local;
use crate::github::upload_image_to_github;
use gloo_file::futures::read_as_bytes;

const MODAL_OVERLAY: &str = "fixed inset-0 bg-black/50 flex justify-center items-center z-[1000]";
const MODAL_CONTENT: &str = "bg-white p-8 rounded-lg w-[90%] max-w-[600px] max-h-[90vh] overflow-y-auto";
const MODAL_HEADER: &str = "flex justify-between items-center mb-4 border-b border-gray-200 pb-2";
const CLOSE_BTN: &str = "bg-gray-400 text-white border-none py-2 px-3 rounded cursor-pointer hover:bg-gray-500";

fn light_req_to_key(lr: &LightRequirement) -> String {
    match lr {
        LightRequirement::Low => "Low".to_string(),
        LightRequirement::Medium => "Medium".to_string(),
        LightRequirement::High => "High".to_string(),
    }
}

fn placement_to_key(p: &Placement) -> String {
    match p {
        Placement::Low => "Low".to_string(),
        Placement::Medium => "Medium".to_string(),
        Placement::High => "High".to_string(),
        Placement::Patio => "Patio".to_string(),
        Placement::OutdoorRack => "OutdoorRack".to_string(),
    }
}

#[component]
pub fn OrchidDetail(
    orchid: Orchid,
    on_close: impl Fn() + 'static + Send + Sync,
    on_update: impl Fn(Orchid) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let (note, set_note) = signal(String::new());
    let (file, set_file) = signal_local::<Option<File>>(None);
    let (orchid_signal, set_orchid_signal) = signal(orchid.clone());
    let (is_syncing, set_is_syncing) = signal(false);

    // Edit mode state
    let (is_editing, set_is_editing) = signal(false);
    let (edit_name, set_edit_name) = signal(orchid.name.clone());
    let (edit_species, set_edit_species) = signal(orchid.species.clone());
    let (edit_water_freq, set_edit_water_freq) = signal(orchid.water_frequency_days.to_string());
    let (edit_light_req, set_edit_light_req) = signal(light_req_to_key(&orchid.light_requirement));
    let (edit_placement, set_edit_placement) = signal(placement_to_key(&orchid.placement));
    let (edit_light_lux, set_edit_light_lux) = signal(orchid.light_lux.clone());
    let (edit_temp_range, set_edit_temp_range) = signal(orchid.temperature_range.clone());
    let (edit_notes, set_edit_notes) = signal(orchid.notes.clone());
    let (edit_conservation, set_edit_conservation) = signal(orchid.conservation_status.clone().unwrap_or_default());

    let format_date = |dt: chrono::DateTime<chrono::Utc>| {
        dt.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string()
    };

    let on_file_change = move |ev: leptos::ev::Event| {
        let target: HtmlInputElement = event_target(&ev);
        if let Some(files) = target.files() {
            if let Some(f) = files.get(0) {
                set_file.set(Some(File::from(f)));
            }
        }
    };

    let on_submit_log = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_is_syncing.set(true);

        let current_note = note.get();
        let current_file = file.get();
        let mut updated_orchid = orchid_signal.get();

        spawn_local(async move {
            let mut image_data_str: Option<String> = None;

            if let Some(f) = current_file {
                // 1. Save to IndexedDB (Local)
                let blob: &web_sys::Blob = f.as_ref();
                match save_image_blob(blob.clone()).await {
                    Ok(id) => {
                        image_data_str = Some(id.to_string());
                    },
                    Err(e) => log::error!("Failed to save image locally: {}", e),
                }

                // 2. Upload to GitHub (if token exists)
                let promise = read_as_bytes(&f);
                match promise.await {
                    Ok(data) => {
                        let timestamp = js_sys::Date::now() as u64;
                        let filename = format!("{}_{}.jpg", updated_orchid.id, timestamp);

                        match upload_image_to_github(filename, data).await {
                            Ok(path) => {
                                image_data_str = Some(path);
                            }
                            Err(e) => {
                                log::error!("GitHub upload failed: {}", e);
                                if let Some(window) = web_sys::window() {
                                    let _ = window.alert_with_message(&format!("Image Upload Failed: {}", e));
                                }
                            }
                        }
                    },
                    Err(e) => log::error!("Failed to read file bytes: {}", e),
                }
            }

            updated_orchid.add_log(current_note, image_data_str);
            set_orchid_signal.set(updated_orchid.clone());
            on_update(updated_orchid.clone());

            set_is_syncing.set(false);
            set_note.set(String::new());
            set_file.set(None);
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
        let place = match edit_placement.get().as_str() {
            "Low" => Placement::Low,
            "High" => Placement::High,
            "Patio" => Placement::Patio,
            "OutdoorRack" => Placement::OutdoorRack,
            _ => Placement::Medium,
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
            placement: place,
            light_lux: edit_light_lux.get(),
            temperature_range: edit_temp_range.get(),
            conservation_status: conservation_opt,
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
        set_edit_placement.set(placement_to_key(&current.placement));
        set_edit_light_lux.set(current.light_lux);
        set_edit_temp_range.set(current.temperature_range);
        set_edit_notes.set(current.notes);
        set_edit_conservation.set(current.conservation_status.unwrap_or_default());
        set_is_editing.set(false);
    };

    view! {
        <div class=MODAL_OVERLAY>
            <div class=MODAL_CONTENT>
                <div class=MODAL_HEADER>
                    <h2>{move || orchid_signal.get().name}</h2>
                    <div class="flex gap-2">
                        <button class="py-2 px-3 text-sm text-white bg-green-600 rounded border-none cursor-pointer hover:bg-green-700"
                            on:click=move |_| {
                                let current = orchid_signal.get();
                                set_edit_name.set(current.name);
                                set_edit_species.set(current.species);
                                set_edit_water_freq.set(current.water_frequency_days.to_string());
                                set_edit_light_req.set(light_req_to_key(&current.light_requirement));
                                set_edit_placement.set(placement_to_key(&current.placement));
                                set_edit_light_lux.set(current.light_lux);
                                set_edit_temp_range.set(current.temperature_range);
                                set_edit_notes.set(current.notes);
                                set_edit_conservation.set(current.conservation_status.unwrap_or_default());
                                set_is_editing.set(true);
                            }
                        >"Edit"</button>
                        <button class="py-2 px-3 text-sm text-white bg-blue-600 rounded border-none cursor-pointer hover:bg-blue-700" on:click=move |_| {
                            if let Some(window) = web_sys::window() {
                                let origin = window.location().origin().unwrap_or_default();
                                let pathname = window.location().pathname().unwrap_or_default();
                                let url = format!("{}{}?id={}", origin, pathname, orchid_signal.get().id);

                                let navigator = window.navigator();
                                let clipboard = navigator.clipboard();
                                let _ = clipboard.write_text(&url);
                                let _ = window.alert_with_message("Deep link copied to clipboard!");
                            }
                        }>"Share"</button>
                        <button class=CLOSE_BTN on:click=move |_| on_close()>"Close"</button>
                    </div>
                </div>
                <div>
                    {move || {
                        if is_editing.get() {
                            view! {
                                <div class="mb-4">
                                    <form on:submit=on_edit_save>
                                        <div class="mb-4">
                                            <label class="block mb-1 font-bold">"Name:"</label>
                                            <input type="text" class="w-full p-2 border border-gray-300 rounded"
                                                prop:value=edit_name
                                                on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                                required
                                            />
                                        </div>
                                        <div class="mb-4">
                                            <label class="block mb-1 font-bold">"Species:"</label>
                                            <input type="text" class="w-full p-2 border border-gray-300 rounded"
                                                prop:value=edit_species
                                                on:input=move |ev| set_edit_species.set(event_target_value(&ev))
                                                required
                                            />
                                        </div>
                                        <div class="mb-4">
                                            <label class="block mb-1 font-bold">"Conservation Status:"</label>
                                            <input type="text" class="w-full p-2 border border-gray-300 rounded"
                                                prop:value=edit_conservation
                                                on:input=move |ev| set_edit_conservation.set(event_target_value(&ev))
                                                placeholder="e.g. CITES II (optional)"
                                            />
                                        </div>
                                        <div class="flex gap-4 mb-4">
                                            <div class="flex-1">
                                                <label class="block mb-1 font-bold">"Water Freq (days):"</label>
                                                <input type="number" class="w-full p-2 border border-gray-300 rounded"
                                                    prop:value=edit_water_freq
                                                    on:input=move |ev| set_edit_water_freq.set(event_target_value(&ev))
                                                    required
                                                />
                                            </div>
                                            <div class="flex-1">
                                                <label class="block mb-1 font-bold">"Light Req:"</label>
                                                <select class="w-full p-2 border border-gray-300 rounded"
                                                    prop:value=edit_light_req
                                                    on:change=move |ev| set_edit_light_req.set(event_target_value(&ev))
                                                >
                                                    <option value="Low">"Low"</option>
                                                    <option value="Medium">"Medium"</option>
                                                    <option value="High">"High"</option>
                                                </select>
                                            </div>
                                        </div>
                                        <div class="flex gap-4 mb-4">
                                            <div class="flex-1">
                                                <label class="block mb-1 font-bold">"Placement:"</label>
                                                <select class="w-full p-2 border border-gray-300 rounded"
                                                    prop:value=edit_placement
                                                    on:change=move |ev| set_edit_placement.set(event_target_value(&ev))
                                                >
                                                    <option value="Low">"Low Light Area"</option>
                                                    <option value="Medium">"Medium Light Area"</option>
                                                    <option value="High">"High Light Area"</option>
                                                    <option value="Patio">"Patio (Outdoors)"</option>
                                                    <option value="OutdoorRack">"Outdoor Rack"</option>
                                                </select>
                                            </div>
                                            <div class="flex-1">
                                                <label class="block mb-1 font-bold">"Light (Lux):"</label>
                                                <input type="text" class="w-full p-2 border border-gray-300 rounded"
                                                    prop:value=edit_light_lux
                                                    on:input=move |ev| set_edit_light_lux.set(event_target_value(&ev))
                                                />
                                            </div>
                                        </div>
                                        <div class="mb-4">
                                            <label class="block mb-1 font-bold">"Temp Range:"</label>
                                            <input type="text" class="w-full p-2 border border-gray-300 rounded"
                                                prop:value=edit_temp_range
                                                on:input=move |ev| set_edit_temp_range.set(event_target_value(&ev))
                                                placeholder="e.g. 18-28C (optional)"
                                            />
                                        </div>
                                        <div class="mb-4">
                                            <label class="block mb-1 font-bold">"Notes:"</label>
                                            <textarea class="w-full p-2 border border-gray-300 rounded"
                                                prop:value=edit_notes
                                                on:input=move |ev| set_edit_notes.set(event_target_value(&ev))
                                                rows="3"
                                            ></textarea>
                                        </div>
                                        <div class="flex gap-2">
                                            <button type="submit" class="py-2 px-4 text-white rounded border-none cursor-pointer bg-primary hover:bg-primary-dark">"Save"</button>
                                            <button type="button" class="py-2 px-4 text-white bg-gray-400 rounded border-none cursor-pointer hover:bg-gray-500" on:click=on_edit_cancel>"Cancel"</button>
                                        </div>
                                    </form>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="mb-4">
                                    <p><strong>"Species: "</strong> {move || orchid_signal.get().species}</p>
                                    {move || orchid_signal.get().conservation_status.map(|status| {
                                        view! { <p class="my-1 italic text-red-700"><strong>"Conservation Status: "</strong> {status}</p> }
                                    })}
                                    <p><strong>"Notes: "</strong> {move || orchid_signal.get().notes}</p>
                                </div>
                            }.into_any()
                        }
                    }}

                    <div class="mb-6">
                        <h3>"Add Entry"</h3>
                        <form on:submit=on_submit_log>
                            <div class="mb-4">
                                <label>"Note:"</label>
                                <textarea
                                    prop:value=note
                                    on:input=move |ev| set_note.set(event_target_value(&ev))
                                    placeholder="Growth update, watering note, etc."
                                ></textarea>
                            </div>
                            <div class="mb-4">
                                <label>"Photo (optional):"</label>
                                <input type="file" accept="image/*" on:change=on_file_change />
                            </div>
                            <button type="submit" class="py-3 px-6 text-base text-white rounded border-none cursor-pointer bg-primary hover:bg-primary-dark" disabled=move || is_syncing.get()>
                                {move || if is_syncing.get() { "Syncing..." } else { "Add Entry" }}
                            </button>
                        </form>
                    </div>

                    <div>
                        <h3>"History"</h3>
                        <div class="pl-4 mt-4 border-l-2 border-primary">
                            <For
                                each=move || {
                                    let mut history = orchid_signal.get().history.clone();
                                    history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                                    history
                                }
                                key=|entry| entry.id
                                children=move |entry| {
                                    let img = entry.image_data.clone();
                                    view! {
                                        <div class="relative mb-6 before:content-[''] before:absolute before:-left-[1.4rem] before:top-[0.2rem] before:w-2.5 before:h-2.5 before:bg-primary before:rounded-full">
                                            <span class="block mb-1 text-xs font-bold text-gray-500">{format_date(entry.timestamp)}</span>
                                            <p class="my-1">{entry.note.clone()}</p>
                                            {img.map(|data| view! { <SmartImage data=data /> })}
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

#[component]
fn SmartImage(data: String) -> impl IntoView {
    let (src, set_src) = signal(String::new());

    Effect::new(move |_| {
        let d = data.clone();
        spawn_local(async move {
            if let Ok(id) = d.parse::<u32>() {
                if let Ok(Some(blob)) = get_image_blob(id).await {
                    if let Ok(url) = Url::create_object_url_with_blob(&blob) {
                        set_src.set(url);
                    }
                }
            } else if let Ok(owner) = LocalStorage::get::<String>("repo_owner") {
                if let Ok(repo) = LocalStorage::get::<String>("repo_name") {
                    let url = format!("https://raw.githubusercontent.com/{}/{}/main/src/data/{}", owner, repo, d);
                    set_src.set(url);
                }
            }
        });
    });

    view! {
        <img src=src class="block mt-2 max-w-full rounded max-h-[300px]" alt="Orchid update" />
    }
}
