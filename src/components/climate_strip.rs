use leptos::prelude::*;
use crate::orchid::{ClimateReading, GrowingZone};
use super::{source_badge, format_time_ago};

// ── Grid-aligned Tailwind class constants ────────────────────────────

/// Outer container with rounded border and relative positioning for the accent bar.
const STRIP_CONTAINER: &str = "overflow-hidden relative mb-4 rounded-xl border bg-surface/80 border-stone-200/60 dark:border-stone-700/60";

/// Left accent bar — green-to-gold gradient matching the botanical aesthetic.
const STRIP_ACCENT: &str = "absolute top-3 bottom-3 left-0 w-[3px] rounded-r bg-gradient-to-b from-primary/60 to-accent/40";

/// Grid column template shared by header and data rows.
const GRID_COLS: &str = "sm:grid sm:grid-cols-[minmax(120px,1fr)_70px_55px_65px_76px] sm:gap-x-3";

/// Header row — hidden on mobile, tiny uppercase muted labels.
const HEADER_ROW: &str = "hidden sm:grid grid-cols-[minmax(120px,1fr)_70px_55px_65px_76px] gap-x-3 items-center py-1.5 pr-4 pl-5 text-[10px] font-bold tracking-widest uppercase text-stone-400 dark:text-stone-500 bg-cream/50 dark:bg-stone-800/30 border-b border-stone-100 dark:border-stone-700/50";

/// Data row base classes — grid on sm+, flex-wrap on mobile.
const DATA_ROW_BASE: &str = "sm:items-center py-2.5 pr-4 pl-5 border-b last:border-b-0 border-stone-100 dark:border-stone-700/50";

/// Zebra stripe for even rows.
const ROW_EVEN: &str = "bg-cream/30 dark:bg-stone-800/20";

/// Desktop-only numeric cell (hidden on mobile).
const CELL_DESKTOP: &str = "hidden sm:block text-right tabular-nums text-sm";

/// Mobile-only compact value row.
const CELL_MOBILE_ROW: &str = "flex sm:hidden gap-3 mt-1 text-sm";

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
        <div class=STRIP_CONTAINER>
            // Left accent bar
            <div class=STRIP_ACCENT></div>

            // Column headers (desktop only)
            <div class=HEADER_ROW>
                <span>"Zone"</span>
                <span class="text-right">"Temp"</span>
                <span class="text-right">"RH%"</span>
                <span class="text-right">"VPD"</span>
                <span class="text-right">"Updated"</span>
            </div>

            // Zones with readings — one grid row each
            {move || {
                let u = unit.get();
                readings.get_value().iter().enumerate().map(|(i, r)| {
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

                    let vpd_str = vpd.map(|v| format!("{:.2}", v)).unwrap_or_default();
                    let humidity_str = format!("{:.0}%", humidity);

                    // Build row class with zebra striping
                    let row_class = if i % 2 == 0 {
                        format!("{GRID_COLS} {DATA_ROW_BASE} {ROW_EVEN}")
                    } else {
                        format!("{GRID_COLS} {DATA_ROW_BASE}")
                    };

                    view! {
                        <div class=row_class>
                            // Zone name + badge (always visible)
                            <div class="flex gap-2 items-center min-w-0">
                                <span class="text-sm font-semibold truncate text-stone-700 dark:text-stone-300">{name}</span>
                                {source_badge(&source)}
                            </div>

                            // Desktop cells — individual grid columns
                            <span class={format!("{CELL_DESKTOP} font-semibold text-primary dark:text-primary-light")}>
                                {temp_val.clone()}<span class="text-xs opacity-50">{temp_unit_label}</span>
                            </span>
                            <span class={format!("{CELL_DESKTOP} text-stone-500 dark:text-stone-400")}>
                                {humidity_str.clone()}
                            </span>
                            <span class={format!("{CELL_DESKTOP} text-stone-400 dark:text-stone-500")}>
                                {vpd_str.clone()}
                            </span>
                            <span class={format!("{CELL_DESKTOP} text-xs text-stone-400 dark:text-stone-500")}>
                                {ago.clone()}
                            </span>

                            // Mobile compact row — all values inline
                            <div class=CELL_MOBILE_ROW>
                                <span class="font-semibold tabular-nums text-primary dark:text-primary-light">
                                    {temp_val}<span class="text-xs opacity-50">{temp_unit_label}</span>
                                </span>
                                <span class="tabular-nums text-stone-500 dark:text-stone-400">{humidity_str}</span>
                                {(!vpd_str.is_empty()).then(|| view! {
                                    <span class="tabular-nums text-stone-400 dark:text-stone-500">"VPD "{vpd_str}</span>
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
                            <div class="flex flex-wrap gap-3 justify-between items-center py-2.5 pr-4 pl-5">
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
