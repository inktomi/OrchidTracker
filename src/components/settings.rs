use leptos::prelude::*;
use gloo_storage::{LocalStorage, Storage};

const MODAL_OVERLAY: &str = "fixed inset-0 bg-black/50 flex justify-center items-center z-[1000]";
const MODAL_HEADER: &str = "flex justify-between items-center mb-4 border-b border-gray-200 pb-2";
const CLOSE_BTN: &str = "bg-gray-400 text-white border-none py-2 px-3 rounded cursor-pointer hover:bg-gray-500";

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

    let on_close_clone = on_close.clone();
    let on_save = move |_| {
        let _ = LocalStorage::set("github_token", token.get());
        let _ = LocalStorage::set("repo_owner", owner.get());
        let _ = LocalStorage::set("repo_name", repo.get());
        let _ = LocalStorage::set("gemini_api_key", gemini_key.get());
        let _ = LocalStorage::set("temp_unit", temp_unit.get());
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

                    <p class="text-sm text-gray-500 bg-gray-100 p-2 rounded mb-4">
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
                    <p class="text-sm text-gray-500 bg-gray-100 p-2 rounded mb-4">"Enter your Gemini API Key to enable image scanning and care suggestions."</p>
                     <div class="mb-4">
                        <label>"Gemini API Key:"</label>
                        <input type="password" prop:value=gemini_key on:input=move |ev| set_gemini_key.set(event_target_value(&ev)) />
                    </div>

                    <div class="mt-4">
                        <button class="bg-primary text-white border-none py-3 px-6 rounded cursor-pointer text-base hover:bg-primary-dark" on:click=on_save>"Save Settings"</button>
                    </div>
                </div>
            </div>
        </div>
    }
}
