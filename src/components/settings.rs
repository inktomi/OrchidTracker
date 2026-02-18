use leptos::prelude::*;
use super::{MODAL_OVERLAY, MODAL_CONTENT, MODAL_HEADER, BTN_PRIMARY, BTN_CLOSE};

#[component]
pub fn SettingsModal(
    on_close: impl Fn(String) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let (temp_unit, set_temp_unit) = signal("C".to_string());

    view! {
        <div class=MODAL_OVERLAY>
            <div class=MODAL_CONTENT>
                <div class=MODAL_HEADER>
                    <h2 class="m-0">"Settings"</h2>
                    <button class=BTN_CLOSE on:click=move |_| on_close(temp_unit.get_untracked())>"Close"</button>
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
                        <button class=BTN_PRIMARY on:click=move |_| on_close(temp_unit.get_untracked())>"Save Settings"</button>
                    </div>
                </div>
            </div>
        </div>
    }
}
