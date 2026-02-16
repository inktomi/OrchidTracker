use leptos::*;
use gloo_storage::{LocalStorage, Storage};
use web_sys::{HtmlInputElement, Event};

#[component]
pub fn SettingsModal<F>(on_close: F) -> impl IntoView
where
    F: Fn() + 'static + Clone,
{
    let (token, set_token) = create_signal("".to_string());
    let (owner, set_owner) = create_signal("".to_string());
    let (repo, set_repo) = create_signal("".to_string());
    let (gemini_key, set_gemini_key) = create_signal("".to_string());
    let (temp_unit, set_temp_unit) = create_signal("C".to_string());

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
        <div class="modal-overlay">
            <div class="modal-content settings-modal">
                <div class="modal-header">
                    <h2>"Sync Settings (GitHub) & AI"</h2>
                    <button class="close-btn" on:click=move |_| on_close()>"X"</button>
                </div>
                <div class="modal-body">
                    <div class="form-group">
                        <label>"Temperature Unit:"</label>
                        <select
                            on:change=move |ev| set_temp_unit.set(event_target_value(&ev))
                            prop:value=temp_unit
                        >
                            <option value="C">"Celsius (°C)"</option>
                            <option value="F">"Fahrenheit (°F)"</option>
                        </select>
                    </div>

                    <p class="settings-hint">
                        "Enter your GitHub Personal Access Token (PAT) to enable syncing changes directly to the repository."
                        <br/>
                        "Requires 'repo' scope."
                    </p>
                    <div class="form-group">
                        <label>"Repo Owner (Username):"</label>
                        <input type="text" prop:value=owner on:input=move |ev| set_owner.set(event_target_value(&ev)) />
                    </div>
                    <div class="form-group">
                        <label>"Repo Name:"</label>
                        <input type="text" prop:value=repo on:input=move |ev| set_repo.set(event_target_value(&ev)) />
                    </div>
                    <div class="form-group">
                        <label>"Personal Access Token:"</label>
                        <input type="password" prop:value=token on:input=move |ev| set_token.set(event_target_value(&ev)) />
                    </div>
                    
                    <hr class="separator" />
                    
                    <h3>"AI Integration (Google Gemini)"</h3>
                    <p class="settings-hint">"Enter your Gemini API Key to enable image scanning and care suggestions."</p>
                     <div class="form-group">
                        <label>"Gemini API Key:"</label>
                        <input type="password" prop:value=gemini_key on:input=move |ev| set_gemini_key.set(event_target_value(&ev)) />
                    </div>

                    <div class="button-group">
                        <button on:click=on_save>"Save Settings"</button>
                    </div>
                </div>
            </div>
        </div>
    }
}
