use leptos::prelude::*;
use crate::orchid::{ClimateReading, GrowingZone};
use super::{source_badge, format_time_ago};

#[component]
pub fn ClimateDashboard(
    readings: Vec<ClimateReading>,
    zones: Vec<GrowingZone>,
    unit: Memo<String>,
    on_show_wizard: impl Fn(GrowingZone) + 'static + Copy + Send + Sync,
    on_zones_changed: impl Fn() + 'static + Copy + Send + Sync,
    temp_unit_str: String,
    #[prop(optional)] read_only: bool,
) -> impl IntoView {
    // Find zones with no readings
    let zone_ids_with_readings: Vec<String> = readings.iter().map(|r| r.zone_id.clone()).collect();
    let empty_zones: Vec<GrowingZone> = zones.into_iter()
        .filter(|z| !zone_ids_with_readings.contains(&z.id))
        .collect();

    if readings.is_empty() && empty_zones.is_empty() {
        return view! { <div></div> }.into_any();
    }

    let readings = StoredValue::new(readings);
    let empty_zones = StoredValue::new(empty_zones);
    let temp_unit_stored = StoredValue::new(temp_unit_str);

    view! {
        <div>
            // Zones with readings
            {move || {
                let u = unit.get();
                readings.get_value().iter().map(|r| {
                    let (temp_val, temp_unit_label) = if u == "F" {
                        let f = (r.temperature * 9.0 / 5.0) + 32.0;
                        (format!("{:.1}", f), "\u{00B0}F")
                    } else {
                        (format!("{:.1}", r.temperature), "\u{00B0}C")
                    };

                    let name = r.zone_name.clone();
                    let humidity = r.humidity;
                    let vpd = r.vpd;
                    let ago = format_time_ago(&r.recorded_at);
                    let source = r.source.clone();

                    view! {
                        <div class="overflow-hidden p-5 pl-6 mx-auto mb-4 rounded-2xl border shadow-sm bg-surface border-stone-200/60 max-w-[700px] climate-card dark:border-stone-700/60">
                            <div class="flex flex-wrap gap-4 justify-between items-start">
                                <div class="flex flex-col gap-1.5">
                                    <div class="flex gap-2.5 items-center">
                                        <h3 class="m-0 text-base font-display text-stone-700 dark:text-stone-300">{name}</h3>
                                        {source_badge(&source)}
                                    </div>
                                    <div class="text-[11px] text-stone-400 dark:text-stone-500">
                                        {ago}
                                    </div>
                                </div>
                                <div class="flex flex-wrap gap-5 items-center">
                                    <div class="flex flex-col items-center climate-value-in">
                                        <span class="font-bold tracking-widest uppercase text-[10px] text-stone-400 dark:text-stone-500">"Temp"</span>
                                        <span class="text-2xl font-display text-primary dark:text-primary-light">{temp_val}</span>
                                        <span class="font-medium text-[10px] text-primary/50 dark:text-primary-light/50">{temp_unit_label}</span>
                                    </div>
                                    <div class="w-px h-8 bg-stone-200 dark:bg-stone-700"></div>
                                    <div class="flex flex-col items-center climate-value-in" style="animation-delay: 0.05s">
                                        <span class="font-bold tracking-widest uppercase text-[10px] text-stone-400 dark:text-stone-500">"Humidity"</span>
                                        <span class="text-2xl font-display text-primary dark:text-primary-light">{format!("{:.0}", humidity)}</span>
                                        <span class="font-medium text-[10px] text-primary/50 dark:text-primary-light/50">"%"</span>
                                    </div>
                                    {vpd.map(|v| view! {
                                        <div class="w-px h-8 bg-stone-200 dark:bg-stone-700"></div>
                                        <div class="flex flex-col items-center climate-value-in" style="animation-delay: 0.1s">
                                            <span class="font-bold tracking-widest uppercase text-[10px] text-stone-400 dark:text-stone-500">"VPD"</span>
                                            <span class="text-2xl font-display text-primary dark:text-primary-light">{format!("{:.2}", v)}</span>
                                            <span class="font-medium text-[10px] text-primary/50 dark:text-primary-light/50">"kPa"</span>
                                        </div>
                                    })}
                                </div>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()
            }}

            // Empty zone prompts
            {move || {
                let zones = empty_zones.get_value();
                if zones.is_empty() {
                    return vec![];
                }
                zones.iter().map(|z| {
                    let zone_for_wizard = z.clone();
                    let zone_for_manual = z.clone();
                    let zone_name = z.name.clone();
                    let (show_manual, set_show_manual) = signal(false);
                    let tu = temp_unit_stored.get_value();

                    view! {
                        <div class="p-5 mx-auto mb-4 rounded-2xl border border-dashed bg-surface/40 border-stone-300/70 max-w-[700px] dark:border-stone-600/60 empty-zone-shimmer">
                            <div class="flex flex-wrap gap-3 justify-between items-center">
                                <div>
                                    <h3 class="m-0 text-base font-display text-stone-400 dark:text-stone-500">{zone_name}</h3>
                                    <p class="mt-1.5 text-xs text-stone-400/80 dark:text-stone-500/80">"No conditions recorded yet"</p>
                                </div>
                                {(!read_only).then(|| view! {
                                    <div class="flex gap-2">
                                        <button
                                            class="flex gap-1.5 items-center py-2 px-3.5 text-xs font-semibold rounded-xl border-none transition-all cursor-pointer text-accent-dark bg-accent/10 dark:text-accent-light dark:bg-accent/10 dark:hover:bg-accent/20 hover:bg-accent/20"
                                            on:click=move |_| on_show_wizard(zone_for_wizard.clone())
                                        >
                                            <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 20 20" fill="currentColor">
                                                <path d="M11.3 1.046A1 1 0 0112 2v5h4a1 1 0 01.82 1.573l-7 10A1 1 0 018 18v-5H4a1 1 0 01-.82-1.573l7-10a1 1 0 011.12-.38z" />
                                            </svg>
                                            "Estimate"
                                        </button>
                                        <button
                                            class="flex gap-1.5 items-center py-2 px-3.5 text-xs font-semibold rounded-xl border-none transition-all cursor-pointer text-sky-600 bg-sky-50 dark:text-sky-400 dark:bg-sky-900/20 dark:hover:bg-sky-900/40 hover:bg-sky-100"
                                            on:click=move |_| set_show_manual.update(|v| *v = !*v)
                                        >
                                            <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 20 20" fill="currentColor">
                                                <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
                                            </svg>
                                            "Log Reading"
                                        </button>
                                    </div>
                                })}
                            </div>
                            {move || show_manual.get().then(|| {
                                let zm = zone_for_manual.clone();
                                let t = tu.clone();
                                view! {
                                    <crate::components::manual_reading::ManualReadingForm
                                        zone=zm
                                        temp_unit=t
                                        on_saved=move || {
                                            on_zones_changed();
                                            set_show_manual.set(false);
                                        }
                                        on_cancel=move || set_show_manual.set(false)
                                    />
                                }
                            })}
                        </div>
                    }
                }).collect::<Vec<_>>()
            }}
        </div>
    }.into_any()
}

