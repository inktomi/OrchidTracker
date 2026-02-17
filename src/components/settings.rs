use leptos::prelude::*;
use leptos::task::spawn_local;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

const MODAL_OVERLAY: &str = "fixed inset-0 bg-black/50 flex justify-center items-center z-[1000]";
const MODAL_HEADER: &str = "flex justify-between items-center mb-4 border-b border-gray-200 pb-2";
const CLOSE_BTN: &str = "bg-gray-400 text-white border-none py-2 px-3 rounded cursor-pointer hover:bg-gray-500";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct ModelInfo {
    name: String,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "supportedGenerationMethods")]
    supported_generation_methods: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ModelList {
    models: Vec<ModelInfo>,
}

#[component]
pub fn SettingsModal<F>(on_close: F) -> impl IntoView
where
    F: Fn() + 'static + Clone + Send + Sync,
{
    let (token, set_token) = signal(String::new());
    let (owner, set_owner) = signal(String::new());
    let (repo, set_repo) = signal(String::new());
    let (gemini_key, set_gemini_key) = signal(String::new());
    let (temp_unit, set_temp_unit) = signal("C".to_string());
    let (gemini_model, set_gemini_model) = signal("gemini-1.5-flash".to_string());
    let (available_models, set_available_models) = signal::<Vec<ModelInfo>>(Vec::new());
    let (fetch_error, set_fetch_error) = signal::<Option<String>>(None);

    // Load initial values
    if let Ok(t) = LocalStorage::get("github_token") {
        set_token.set(t);
    }
    if let Ok(o) = LocalStorage::get("repo_owner") {
        set_owner.set(o);
    }
    if let Ok(r) = LocalStorage::get("repo_name") {
        set_repo.set(r);
    }
    if let Ok(k) = LocalStorage::get("gemini_api_key") {
        set_gemini_key.set(k);
    }
    if let Ok(u) = LocalStorage::get("temp_unit") {
        set_temp_unit.set(u);
    } else {
        set_temp_unit.set("C".to_string());
    }
    if let Ok(m) = LocalStorage::get("gemini_model") {
        set_gemini_model.set(m);
    }

    let fetch_models = move || {
        let key = gemini_key.get();
        if key.is_empty() { return; }
        
        spawn_local(async move {
            let url = format!("https://generativelanguage.googleapis.com/v1beta/models?key={}", key.trim());
            match reqwest::get(&url).await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        if let Ok(list) = resp.json::<ModelList>().await {
                            // Filter for 'generateContent' support
                            let filtered: Vec<ModelInfo> = list.models.into_iter()
                                .filter(|m| m.supported_generation_methods.as_ref()
                                    .map_or(false, |methods| methods.iter().any(|method| method == "generateContent")))
                                .collect();
                            set_available_models.set(filtered);
                            set_fetch_error.set(None);
                        } else {
                            set_fetch_error.set(Some("Failed to parse models".into()));
                        }
                    } else {
                        set_fetch_error.set(Some(format!("Error fetching models: {}", resp.status())));
                    }
                },
                Err(e) => set_fetch_error.set(Some(format!("Network error: {}", e))),
            }
        });
    };

    // Auto-fetch on mount if key exists
    Effect::new(move |_| {
        if !gemini_key.get_untracked().is_empty() {
            fetch_models();
        }
    });

    let on_close_clone = on_close.clone();
    let on_save = move |_| {
        let _ = LocalStorage::set("github_token", token.get());
        let _ = LocalStorage::set("repo_owner", owner.get());
        let _ = LocalStorage::set("repo_name", repo.get());
        let _ = LocalStorage::set("gemini_api_key", gemini_key.get());
        let _ = LocalStorage::set("temp_unit", temp_unit.get());
        let _ = LocalStorage::set("gemini_model", gemini_model.get());
        on_close_clone();
    };

    view! {
        <div class=MODAL_OVERLAY>
            <div class="bg-white p-8 rounded-lg w-[90%] max-w-[500px] max-h-[90vh] overflow-y-auto">
                <div class=MODAL_HEADER>
                    <h2>"Sync Settings (GitHub) & AI"</h2>
                    <button class=CLOSE_BTN on:click=move |_| on_close()>"X"</button>
                </div>
                <div>
                    <div class="mb-4">
                        <label>"Temperature Unit:"</label>
                        <select
                            on:change=move |ev| set_temp_unit.set(event_target_value(&ev))
                            prop:value=temp_unit
                        >
                            <option value="C">"Celsius (C)"</option>
                            <option value="F">"Fahrenheit (F)"</option>
                        </select>
                    </div>

                    <p class="p-2 mb-4 text-sm text-gray-500 bg-gray-100 rounded">
                        "Enter your GitHub Personal Access Token (PAT) to enable syncing changes directly to the repository."
                        <br/>
                        "Required scopes: " <strong>"repo"</strong> " (for private repos) or " <strong>"public_repo"</strong> " (for public repos)."
                    </p>
                    <div class="mb-4">
                        <label>"Repo Owner (Username):"</label>
                        <input type="text" prop:value=owner on:input=move |ev| set_owner.set(event_target_value(&ev)) />
                    </div>
                    <div class="mb-4">
                        <label>"Repo Name:"</label>
                        <input type="text" prop:value=repo on:input=move |ev| set_repo.set(event_target_value(&ev)) />
                    </div>
                    <div class="mb-4">
                        <label>"Personal Access Token:"</label>
                        <input type="password" prop:value=token on:input=move |ev| set_token.set(event_target_value(&ev)) />
                    </div>

                    <hr class="my-4 border-gray-200" />

                    <h3>"AI Integration (Google Gemini)"</h3>
                    <p class="p-2 mb-4 text-sm text-gray-500 bg-gray-100 rounded">"Enter your Gemini API Key to enable image scanning and care suggestions."</p>
                     <div class="mb-4">
                        <label>"Gemini API Key:"</label>
                        <input type="password" prop:value=gemini_key on:input=move |ev| set_gemini_key.set(event_target_value(&ev)) on:blur=move |_| fetch_models() />
                    </div>
                    
                    <div class="mb-4">
                        <label>"Gemini Model:"</label>
                        <div class="flex gap-2">
                            <select
                                class="flex-grow"
                                on:change=move |ev| set_gemini_model.set(event_target_value(&ev))
                                prop:value=gemini_model
                            >
                                <option value="" disabled>"Select a model"</option>
                                <For
                                    each=move || available_models.get()
                                    key=|m| m.name.clone()
                                    children=move |m| {
                                        let name = m.name.replace("models/", "");
                                        let label = m.display_name.unwrap_or(name.clone());
                                        view! { <option value=name>{label}</option> }
                                    }
                                />
                                // Fallback option if list empty
                                {move || if available_models.get().is_empty() {
                                    view! { <option value="gemini-1.5-flash">"gemini-1.5-flash (Default)"</option> }.into_any()
                                } else {
                                    view! {}.into_any()
                                }}
                            </select>
                            <button class="bg-gray-200 border border-gray-300 rounded px-3 hover:bg-gray-300" on:click=move |_| fetch_models()>"Refresh"</button>
                        </div>
                        {move || fetch_error.get().map(|err| view! { <p class="text-xs text-red-500 mt-1">{err}</p> })}
                    </div>

                    <div class="mt-4">
                        <button class="py-3 px-6 text-base text-white rounded border-none cursor-pointer bg-primary hover:bg-primary-dark" on:click=on_save>"Save Settings"</button>
                    </div>
                </div>
            </div>
        </div>
    }
}
