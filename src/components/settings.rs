use leptos::prelude::*;
use super::{MODAL_OVERLAY, MODAL_CONTENT, MODAL_HEADER, BTN_PRIMARY, BTN_CLOSE};

#[component]
pub fn SettingsModal<F>(on_close: F) -> impl IntoView
where
    F: Fn() + 'static + Clone + Send + Sync,
{
    let (temp_unit, set_temp_unit) = signal("C".to_string());

    let on_close_clone = on_close.clone();
    let on_save = move |_| {
        // temp_unit is passed back via on_close callback in the parent
        on_close_clone();
    };

    view! {
        <div class=MODAL_OVERLAY>
            <div class=MODAL_CONTENT>
                <div class=MODAL_HEADER>
                    <h2 class="m-0">"Settings"</h2>
                    <button class=BTN_CLOSE on:click=move |_| on_close()>"Close"</button>
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

                    <p class="p-3 mb-4 text-xs leading-relaxed rounded-lg text-stone-500 bg-secondary dark:text-stone-400">
                        "API keys and sync settings are managed server-side. Contact your administrator to update them."
                    </p>

                    <div class="mt-6">
                        <button class=BTN_PRIMARY on:click=on_save>"Save Settings"</button>
                    </div>
                </div>
            </div>
        </div>
    }
}
