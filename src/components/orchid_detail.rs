use leptos::prelude::*;
use leptos::task::spawn_local;
use gloo_file::File;
use gloo_storage::{LocalStorage, Storage};
use web_sys::{HtmlInputElement, Url};
use crate::orchid::Orchid;
use crate::db::{save_image_blob, get_image_blob};
use chrono::Local;
use crate::github::upload_image_to_github;
use gloo_file::futures::read_as_bytes;

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

    view! {
        <div class="modal-overlay">
            <div class="modal-content">
                <div class="modal-header">
                    <h2>{move || orchid_signal.get().name}</h2>
                    <div class="header-actions">
                        <button class="share-btn" on:click=move |_| {
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
                        <button class="close-btn" on:click=move |_| on_close()>"Close"</button>
                    </div>
                </div>
                <div class="modal-body">
                    <div class="detail-info">
                        <p><strong>"Species: "</strong> {move || orchid_signal.get().species}</p>
                        {move || orchid_signal.get().conservation_status.map(|status| {
                            view! { <p class="conservation-status"><strong>"Conservation Status: "</strong> {status}</p> }
                        })}
                        <p><strong>"Notes: "</strong> {move || orchid_signal.get().notes}</p>
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
                            <button type="submit" disabled=move || is_syncing.get()>
                                {move || if is_syncing.get() { "Syncing..." } else { "Add Entry" }}
                            </button>
                        </form>
                    </div>

                    <div class="history-section">
                        <h3>"History"</h3>
                        <div class="timeline">
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
                                        <div class="timeline-entry">
                                            <span class="entry-date">{format_date(entry.timestamp)}</span>
                                            <p class="entry-note">{entry.note.clone()}</p>
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
            } else {
                if let Ok(owner) = LocalStorage::get::<String>("repo_owner") {
                    if let Ok(repo) = LocalStorage::get::<String>("repo_name") {
                        let url = format!("https://raw.githubusercontent.com/{}/{}/main/src/data/{}", owner, repo, d);
                        set_src.set(url);
                    }
                }
            }
        });
    });

    view! {
        <img src=src class="timeline-image" alt="Orchid update" />
    }
}
