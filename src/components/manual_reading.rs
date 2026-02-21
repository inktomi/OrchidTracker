use leptos::prelude::*;
use crate::orchid::GrowingZone;
use super::BTN_PRIMARY;

const INPUT_MR: &str = "w-full px-3 py-2 text-sm bg-white/80 border border-stone-300/50 rounded-lg outline-none transition-all duration-200 placeholder:text-stone-400 focus:bg-white focus:border-primary/40 focus:ring-2 focus:ring-primary/10 dark:bg-stone-800/80 dark:border-stone-600/50 dark:placeholder:text-stone-500 dark:focus:bg-stone-800 dark:focus:border-primary-light/40 dark:focus:ring-primary-light/10";
const LABEL_MR: &str = "block mb-1 text-xs font-semibold tracking-wider uppercase text-stone-400 dark:text-stone-500";

/// Compact inline form for logging a manual climate reading.
#[component]
pub fn ManualReadingForm(
    zone: GrowingZone,
    temp_unit: String,
    on_saved: impl Fn() + 'static + Copy + Send + Sync,
    on_cancel: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let (temperature, set_temperature) = signal(String::new());
    let (humidity, set_humidity) = signal(String::new());
    let (is_saving, set_is_saving) = signal(false);
    let (error_msg, set_error_msg) = signal::<Option<String>>(None);

    let is_f = temp_unit == "F";
    let zone_stored = StoredValue::new(zone);

    let save = move |_| {
        let temp_str = temperature.get();
        let hum_str = humidity.get();

        let temp_val: f64 = match temp_str.parse() {
            Ok(v) => v,
            Err(_) => { set_error_msg.set(Some("Invalid temperature".into())); return; }
        };
        let hum_val: f64 = match hum_str.parse() {
            Ok(v) => v,
            Err(_) => { set_error_msg.set(Some("Invalid humidity".into())); return; }
        };

        let temp_c = if is_f { crate::estimation::f_to_c(temp_val) } else { temp_val };

        if !(0.0..=100.0).contains(&hum_val) {
            set_error_msg.set(Some("Humidity must be 0-100%".into()));
            return;
        }

        set_is_saving.set(true);
        set_error_msg.set(None);
        let z = zone_stored.get_value();

        leptos::task::spawn_local(async move {
            match crate::server_fns::climate::log_manual_reading(
                z.id.clone(), z.name.clone(), temp_c, hum_val,
            ).await {
                Ok(()) => on_saved(),
                Err(e) => {
                    log::error!("Failed to log manual reading: {}", e);
                    set_error_msg.set(Some("Failed to save reading".into()));
                }
            }
            set_is_saving.set(false);
        });
    };

    view! {
        <div class="p-3 mt-2 rounded-xl border animate-fade-in bg-stone-50/80 border-stone-200/60 dark:bg-stone-800/50 dark:border-stone-700/60">
            <div class="flex gap-3 items-end">
                <div class="flex-1">
                    <label class=LABEL_MR>{if is_f { "Temp (°F)" } else { "Temp (°C)" }}</label>
                    <input type="number" class=INPUT_MR step="0.1"
                        placeholder=if is_f { "72" } else { "22" }
                        prop:value=temperature
                        on:input=move |ev| set_temperature.set(event_target_value(&ev))
                    />
                </div>
                <div class="flex-1">
                    <label class=LABEL_MR>"Humidity (%)"</label>
                    <input type="number" class=INPUT_MR step="1" min="0" max="100"
                        placeholder="50"
                        prop:value=humidity
                        on:input=move |ev| set_humidity.set(event_target_value(&ev))
                    />
                </div>
                <div class="flex flex-shrink-0 gap-1.5">
                    <button class=BTN_PRIMARY
                        disabled=move || is_saving.get() || temperature.get().is_empty() || humidity.get().is_empty()
                        on:click=save
                    >{move || if is_saving.get() { "..." } else { "Log" }}</button>
                    <button
                        class="py-2 px-3 text-sm rounded-lg border-none transition-colors cursor-pointer text-stone-400 bg-stone-100 dark:bg-stone-700 dark:hover:bg-stone-600 hover:bg-stone-200"
                        on:click=move |_| on_cancel()
                    >"X"</button>
                </div>
            </div>
            {move || error_msg.get().map(|msg| {
                view! { <p class="mt-2 text-xs text-red-600 dark:text-red-400">{msg}</p> }
            })}
        </div>
    }.into_any()
}
