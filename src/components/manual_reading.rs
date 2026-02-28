use leptos::prelude::*;
use crate::orchid::GrowingZone;

const INPUT_MR: &str = "w-full px-3 py-2 text-sm bg-white/60 border border-stone-200/80 rounded-xl outline-none transition-all duration-200 placeholder:text-stone-400 focus:bg-white focus:border-sky-400/40 focus:ring-2 focus:ring-sky-400/10 dark:bg-stone-800/60 dark:border-stone-600/60 dark:placeholder:text-stone-500 dark:focus:bg-stone-800 dark:focus:border-sky-400/40 dark:focus:ring-sky-400/10";
const LABEL_MR: &str = "block mb-1 text-[10px] font-bold tracking-widest uppercase text-stone-400 dark:text-stone-500";

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
                    tracing::error!("Failed to log manual reading: {}", e);
                    #[cfg(feature = "hydrate")]
                    crate::server_fns::telemetry::emit_error("manual_reading.save", &format!("Failed to log manual reading: {}", e), &[("zone_id", z.id.as_str())]);
                    set_error_msg.set(Some("Failed to save reading".into()));
                }
            }
            set_is_saving.set(false);
        });
    };

    view! {
        <div class="overflow-hidden relative p-3.5 mt-3 rounded-xl border animate-fade-in bg-sky-50/40 border-sky-200/40 dark:bg-sky-900/10 dark:border-sky-800/30">
            // Accent line at top
            <div class="absolute top-0 right-0 left-0 h-0.5 bg-gradient-to-r to-transparent from-sky-400/40 via-sky-300/20"></div>

            <div class="flex gap-3 items-end">
                <div class="flex-1">
                    <label class=LABEL_MR>{if is_f { "Temp (\u{00B0}F)" } else { "Temp (\u{00B0}C)" }}</label>
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
                    <button
                        class="py-2 px-4 text-sm font-semibold text-white rounded-xl border-none shadow-sm transition-all cursor-pointer disabled:opacity-40 bg-sky-500 hover:bg-sky-600"
                        disabled=move || is_saving.get() || temperature.get().is_empty() || humidity.get().is_empty()
                        on:click=save
                    >{move || if is_saving.get() { "..." } else { "Log" }}</button>
                    <button
                        class="flex justify-center items-center w-9 h-9 rounded-xl border-none transition-colors cursor-pointer text-stone-400 bg-stone-100/80 dark:bg-stone-700/50 dark:hover:bg-stone-600 dark:hover:text-stone-300 hover:bg-stone-200 hover:text-stone-600"
                        on:click=move |_| on_cancel()
                        aria-label="Cancel"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 20 20" fill="currentColor">
                            <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                        </svg>
                    </button>
                </div>
            </div>
            {move || error_msg.get().map(|msg| {
                view! { <p class="mt-2 text-xs font-medium text-red-600 dark:text-red-400">{msg}</p> }
            })}
        </div>
    }.into_any()
}
