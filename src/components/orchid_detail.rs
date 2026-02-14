use leptos::*;
use gloo_file::{File, Blob};
use web_sys::{HtmlInputElement, Url};
use crate::orchid::{Orchid, LogEntry};
use crate::db::{save_image_blob, get_image_blob};
use wasm_bindgen::JsCast;
use chrono::Local;

#[component]
pub fn OrchidDetail<F, G>(
    orchid: Orchid,
    on_close: F,
    on_update: G,
) -> impl IntoView
where
    F: Fn() + 'static,
    G: Fn(Orchid) + 'static + Copy,
{
    let (note, set_note) = create_signal("".to_string());
    let (file, set_file) = create_signal::<Option<File>>(None);
    let (orchid_signal, set_orchid_signal) = create_signal(orchid.clone());

    // Helper to format date
    let format_date = |dt: chrono::DateTime<chrono::Utc>| {
        dt.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string()
    };

    let on_file_change = move |ev: leptos::ev::Event| {
        let target = event_target::<HtmlInputElement>(&ev);
        if let Some(files) = target.files() {
            if let Some(f) = files.get(0) {
                set_file.set(Some(File::from(f)));
            }
        }
    };

    let on_submit_log = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        
        let current_note = note.get();
        let current_file = file.get();

        // If we have a file, save it to IndexedDB first
        spawn_local(async move {
            let image_data_str = if let Some(f) = current_file {
                // Convert File to Blob (gloo_file::File derefs to Blob)
                let blob: &web_sys::Blob = f.as_ref();
                match save_image_blob(blob.clone()).await {
                    Ok(id) => Some(id.to_string()),
                    Err(e) => {
                        log::error!("Failed to save image: {}", e);
                        None
                    }
                }
            } else {
                None
            };

            // Update Orchid state
            set_orchid_signal.update(|o| {
                o.add_log(current_note, image_data_str);
            });
            
            // Notify parent to save to LocalStorage
            on_update(orchid_signal.get());

            // Reset form
            set_note.set("".to_string());
            set_file.set(None);
        });
    };

    view! {
        <div class="modal-overlay">
            <div class="modal-content">
                <div class="modal-header">
                    <h2>{orchid_signal.get().name}</h2>
                    <button class="close-btn" on:click=move |_| on_close()>"Close"</button>
                </div>
                <div class="modal-body">
                    <div class="detail-info">
                        <p><strong>"Species: "</strong> {orchid_signal.get().species}</p>
                        <p><strong>"Notes: "</strong> {orchid_signal.get().notes}</p>
                    </div>

                    <div class="add-log-section">
                        <h3>"Add Entry"</h3>
                        <form on:submit=on_submit_log>
                            <div class="form-group">
                                <label>"Note:"</label>
                                <textarea
                                    prop:value=note
                                    on:input=move |ev| set_note.set(event_target_value(&ev))
                                    placeholder="Growth update, watering note, etc."
                                ></textarea>
                            </div>
                            <div class="form-group">
                                <label>"Photo (optional):"</label>
                                <input type="file" accept="image/*" on:change=on_file_change />
                            </div>
                            <button type="submit">"Add Entry"</button>
                        </form>
                    </div>

                    <div class="history-section">
                        <h3>"History"</h3>
                        <div class="timeline">
                            <For
                                each=move || {
                                    let mut history = orchid_signal.get().history.clone();
                                    history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)); // Newest first
                                    history
                                }
                                key=|entry| entry.id
                                children=move |entry| {
                                    view! {
                                        <div class="timeline-entry">
                                            <span class="entry-date">{format_date(entry.timestamp)}</span>
                                            <p class="entry-note">{entry.note}</p>
                                            {
                                                if let Some(img_id_str) = entry.image_data {
                                                    if let Ok(id) = img_id_str.parse::<u32>() {
                                                        view! { <AsyncImage id=id /> }.into_view()
                                                    } else {
                                                        view! { <span>"Invalid Image ID"</span> }.into_view()
                                                    }
                                                } else {
                                                    view! {}.into_view()
                                                }
                                            }
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
fn AsyncImage(id: u32) -> impl IntoView {
    let (src, set_src) = create_signal("".to_string());

    create_effect(move |_| {
        spawn_local(async move {
            if let Ok(Some(blob)) = get_image_blob(id).await {
                if let Ok(url) = Url::create_object_url_with_blob(&blob) {
                    set_src.set(url);
                }
            }
        });
    });

    view! {
        <img src=src class="timeline-image" alt="Orchid update" />
    }
}
