use leptos::prelude::*;
use crate::orchid::{ClimateReading, GrowingZone};

const BADGE_ESTIMATED: &str = "inline-flex py-0.5 px-2 text-[10px] font-semibold rounded-full bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-300";
const BADGE_MANUAL: &str = "inline-flex py-0.5 px-2 text-[10px] font-semibold rounded-full bg-sky-100 text-sky-700 dark:bg-sky-900/30 dark:text-sky-300";

#[component]
pub fn ClimateDashboard(
    readings: Vec<ClimateReading>,
    zones: Vec<GrowingZone>,
    unit: Memo<String>,
    on_show_wizard: impl Fn(GrowingZone) + 'static + Copy + Send + Sync,
    on_zones_changed: impl Fn() + 'static + Copy + Send + Sync,
    temp_unit_str: String,
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
                        (format!("{:.1}", f), "F")
                    } else {
                        (format!("{:.1}", r.temperature), "C")
                    };

                    let name = r.zone_name.clone();
                    let humidity = r.humidity;
                    let vpd = r.vpd;
                    let ago = format_time_ago(&r.recorded_at);
                    let source = r.source.clone();

                    view! {
                        <div class="p-4 mx-auto mb-6 rounded-xl border shadow-sm bg-surface border-stone-200 max-w-[700px] dark:border-stone-700">
                            <div class="flex flex-wrap gap-4 justify-between items-center">
                                <div class="flex gap-2 items-center">
                                    <h3 class="m-0 text-base text-stone-700 dark:text-stone-300">{name}</h3>
                                    {source_badge(&source)}
                                </div>
                                <div class="flex flex-wrap gap-6 items-center">
                                    <div class="flex flex-col items-center">
                                        <span class="text-xs font-medium tracking-wider uppercase text-stone-400">"Temp"</span>
                                        <span class="text-xl font-semibold text-primary">{temp_val}" "{temp_unit_label}</span>
                                    </div>
                                    <div class="flex flex-col items-center">
                                        <span class="text-xs font-medium tracking-wider uppercase text-stone-400">"Humidity"</span>
                                        <span class="text-xl font-semibold text-primary">{format!("{:.1}", humidity)}"%"</span>
                                    </div>
                                    {vpd.map(|v| view! {
                                        <div class="flex flex-col items-center">
                                            <span class="text-xs font-medium tracking-wider uppercase text-stone-400">"VPD"</span>
                                            <span class="text-xl font-semibold text-primary">{format!("{:.2}", v)}" kPa"</span>
                                        </div>
                                    })}
                                </div>
                            </div>
                            <div class="mt-2 text-xs text-right text-stone-400">
                                "Updated: " {ago}
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
                        <div class="p-4 mx-auto mb-6 rounded-xl border border-dashed shadow-sm bg-surface/50 border-stone-300 max-w-[700px] dark:border-stone-600">
                            <div class="flex flex-wrap gap-3 justify-between items-center">
                                <div>
                                    <h3 class="m-0 text-base text-stone-500 dark:text-stone-400">{zone_name}</h3>
                                    <p class="mt-1 text-xs text-stone-400">"No conditions recorded yet"</p>
                                </div>
                                <div class="flex gap-2">
                                    <button
                                        class="py-1.5 px-3 text-xs font-semibold text-amber-600 bg-amber-50 rounded-lg border-none transition-colors cursor-pointer dark:text-amber-400 hover:bg-amber-100 dark:bg-amber-900/20 dark:hover:bg-amber-900/40"
                                        on:click=move |_| on_show_wizard(zone_for_wizard.clone())
                                    >"Estimate Conditions"</button>
                                    <button
                                        class="py-1.5 px-3 text-xs font-semibold rounded-lg border-none transition-colors cursor-pointer text-sky-600 bg-sky-50 dark:text-sky-400 dark:bg-sky-900/20 dark:hover:bg-sky-900/40 hover:bg-sky-100"
                                        on:click=move |_| set_show_manual.update(|v| *v = !*v)
                                    >"Log Reading"</button>
                                </div>
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

fn source_badge(source: &Option<String>) -> Option<leptos::tachys::view::any_view::AnyView> {
    match source.as_deref() {
        Some("wizard") => Some(leptos::IntoView::into_view(
            leptos::view! { <span class=BADGE_ESTIMATED>"Estimated"</span> }
        ).into_any()),
        Some("manual") => Some(leptos::IntoView::into_view(
            leptos::view! { <span class=BADGE_MANUAL>"Manual"</span> }
        ).into_any()),
        _ => None,
    }
}

fn format_time_ago(recorded_at: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(*recorded_at);

    if diff.num_minutes() < 1 {
        "just now".to_string()
    } else if diff.num_minutes() < 60 {
        format!("{} min ago", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{} hr ago", diff.num_hours())
    } else {
        format!("{} days ago", diff.num_days())
    }
}
