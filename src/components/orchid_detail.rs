use leptos::*;
use gloo_storage::{LocalStorage, Storage};
use gloo_file::{File, Blob};
use web_sys::{HtmlInputElement, Url, FileReader};
use crate::orchid::{Orchid, LogEntry};
use crate::db::{save_image_blob, get_image_blob};
use wasm_bindgen::JsCast;
use chrono::Local;
use crate::github::{upload_image_to_github, sync_orchids_to_github};
use gloo_file::futures::read_as_bytes;

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
    let (is_syncing, set_is_syncing) = create_signal(false);

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
                // We need to read the file as bytes
                let promise = read_as_bytes(&f);
                match promise.await {
                    Ok(data) => {
                        let timestamp = js_sys::Date::now() as u64;
                        let filename = format!("{}_{}.jpg", updated_orchid.id, timestamp);
                        
                        match upload_image_to_github(filename.clone(), data).await {
                            Ok(path) => {
                                // If GitHub upload succeeds, we could store the path instead of ID?
                                // Ideally we store both or handle migration.
                                // For now, let's store the local ID, but we *also* want the remote path in orchids.json
                                // Wait, `orchids.json` is shared. It should have the REMOTE path.
                                // But local user needs IndexedDB for speed/offline?
                                // Let's store the REMOTE path in the LogEntry if sync succeeds.
                                image_data_str = Some(path);
                            }
                            Err(e) => log::error!("GitHub upload failed: {}", e),
                        }
                    },
                    Err(e) => log::error!("Failed to read file bytes: {}", e),
                }
            }

            // Update Orchid state
            updated_orchid.add_log(current_note, image_data_str);
            set_orchid_signal.set(updated_orchid.clone());
            
            // Notify parent (LocalStorage update)
            on_update(updated_orchid.clone());
            
            // Sync entire JSON to GitHub
            // Note: This requires the parent to pass the FULL list, or we fetch it?
            // Actually `on_update` updates the list in App.
            // But we don't have access to the full list here to push it.
            // Strategy: We can't easily push the full JSON from this child component without prop drilling the whole list.
            // Simplified: We rely on the App component to handle the sync? Or we accept that this component
            // only updates local state, and we add a "Sync" button in the App header?
            
            // Re-think: User wants sync.
            // Ideally `on_update` handles the sync.
            
            set_is_syncing.set(false);
            set_note.set("".to_string());
            set_file.set(None);
        });
    };

    view! {
        <div class="modal-overlay">
            <div class="modal-content">
                <div class="modal-header">
                    <h2>{orchid_signal.get().name}</h2>
                    <div class="header-actions">
                        <button class="share-btn" on:click=move |_| {
                            if let Some(window) = web_sys::window() {
                                let origin = window.location().origin().unwrap_or_default();
                                let pathname = window.location().pathname().unwrap_or_default();
                                let url = format!("{}{}?id={}", origin, pathname, orchid_signal.get().id);
                                
                                let navigator = window.navigator();
                                if let Some(clipboard) = navigator.clipboard() {
                                    let _ = clipboard.write_text(&url);
                                    let _ = window.alert_with_message("Deep link copied to clipboard!");
                                }
                            }
                        }>"🔗 Share"</button>
                        <button class="close-btn" on:click=move |_| on_close()>"Close"</button>
                    </div>
                </div>
                <div class="modal-body">
                    <div class="detail-info">
                        <p><strong>"Species: "</strong> {orchid_signal.get().species}</p>
                         {if let Some(status) = orchid_signal.get().conservation_status {
                            view! { <p class="conservation-status"><strong>"Conservation Status: "</strong> {status}</p> }.into_view()
                        } else {
                            view! {}.into_view()
                        }}
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
                                                if let Some(img_data) = entry.image_data {
                                                    view! { <SmartImage data=img_data /> }.into_view()
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
fn SmartImage(data: String) -> impl IntoView {
    // Determine if data is an ID (local) or a Path (remote)
    // If it parses as u32, it's local ID.
    // If it contains "images/", it's remote path.
    
    let (src, set_src) = create_signal("".to_string());

    create_effect(move |_| {
        let d = data.clone();
        spawn_local(async move {
            if let Ok(id) = d.parse::<u32>() {
                // Local IndexedDB
                if let Ok(Some(blob)) = get_image_blob(id).await {
                    if let Ok(url) = Url::create_object_url_with_blob(&blob) {
                        set_src.set(url);
                    }
                }
            } else {
                // Remote GitHub Path
                // e.g. "images/123_456.jpg"
                // Construct raw URL: https://raw.githubusercontent.com/{owner}/{repo}/main/src/data/images/{filename}
                // Or if deployed: relative path might work if image is copied to dist.
                // Let's assume we uploaded to `src/data/images/...`.
                // Trunk does NOT copy `src/data` to dist by default unless configured.
                // We should probably rely on raw.githubusercontent for now as it's easiest for "Sync" logic.
                // We need owner/repo from settings.
                
                if let Ok(owner) = gloo_storage::LocalStorage::get::<String>("repo_owner") {
                    if let Ok(repo) = gloo_storage::LocalStorage::get::<String>("repo_name") {
                        // Raw content URL
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
