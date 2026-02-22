use leptos::prelude::*;
use crate::orchid::{ClimateReading, GrowingZone};
use super::{source_badge, format_time_ago};

/// Compact one-row-per-zone climate strip for the My Plants tab.
#[component]
pub fn ClimateStrip(
    readings: Vec<ClimateReading>,
    zones: Vec<GrowingZone>,
    unit: Memo<String>,
    on_show_wizard: impl Fn(GrowingZone) + 'static + Copy + Send + Sync,
    on_zones_changed: impl Fn() + 'static + Copy + Send + Sync,
    temp_unit_str: String,
) -> impl IntoView {
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
        <div class="overflow-hidden mb-4 rounded-xl border bg-surface/80 border-stone-200/60 dark:border-stone-700/60">
            // Zones with readings — one compact row each
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
                        <div class="flex flex-wrap gap-3 justify-between items-center py-2.5 px-4 border-b last:border-b-0 border-stone-100 dark:border-stone-700/50">
                            <div class="flex gap-2 items-center min-w-0">
                                <span class="text-sm font-semibold truncate text-stone-700 dark:text-stone-300">{name}</span>
                                {source_badge(&source)}
                            </div>
                            <div class="flex gap-4 items-center text-sm">
                                <span class="font-semibold tabular-nums text-primary dark:text-primary-light">{temp_val}<span class="text-xs opacity-50">{temp_unit_label}</span></span>
                                <span class="tabular-nums text-stone-500 dark:text-stone-400">{format!("{:.0}%", humidity)}</span>
                                {vpd.map(|v| view! {
                                    <span class="tabular-nums text-stone-400 dark:text-stone-500">"VPD "{format!("{:.2}", v)}</span>
                                })}
                                <span class="text-xs text-stone-400 dark:text-stone-500">{ago}</span>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()
            }}

            // Empty zones — compact row with action buttons
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
                        <div class="border-b border-dashed last:border-b-0 border-stone-200/60 dark:border-stone-600/40">
                            <div class="flex flex-wrap gap-3 justify-between items-center py-2.5 px-4">
                                <span class="text-sm font-medium text-stone-400 dark:text-stone-500">{zone_name}</span>
                                <div class="flex gap-2 items-center">
                                    <span class="text-xs text-stone-300 dark:text-stone-600">"No data"</span>
                                    <button
                                        class="py-1 px-2.5 font-semibold rounded-lg border-none transition-colors cursor-pointer text-[11px] text-accent-dark bg-accent/10 dark:text-accent-light dark:bg-accent/10 dark:hover:bg-accent/20 hover:bg-accent/20"
                                        on:click=move |_| on_show_wizard(zone_for_wizard.clone())
                                    >
                                        "Estimate"
                                    </button>
                                    <button
                                        class="py-1 px-2.5 font-semibold rounded-lg border-none transition-colors cursor-pointer text-[11px] text-sky-600 bg-sky-50 dark:text-sky-400 dark:bg-sky-900/20 dark:hover:bg-sky-900/40 hover:bg-sky-100"
                                        on:click=move |_| set_show_manual.update(|v| *v = !*v)
                                    >
                                        "Log"
                                    </button>
                                </div>
                            </div>
                            {move || show_manual.get().then(|| {
                                let zm = zone_for_manual.clone();
                                let t = tu.clone();
                                view! {
                                    <div class="px-4 pb-3">
                                        <crate::components::manual_reading::ManualReadingForm
                                            zone=zm
                                            temp_unit=t
                                            on_saved=move || {
                                                on_zones_changed();
                                                set_show_manual.set(false);
                                            }
                                            on_cancel=move || set_show_manual.set(false)
                                        />
                                    </div>
                                }
                            })}
                        </div>
                    }
                }).collect::<Vec<_>>()
            }}
        </div>
    }.into_any()
}
