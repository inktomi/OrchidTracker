use leptos::prelude::*;
use crate::orchid::ClimateReading;

#[component]
pub fn ClimateDashboard(readings: Vec<ClimateReading>, unit: Memo<String>) -> impl IntoView {
    if readings.is_empty() {
        view! { <div class="flex justify-center p-4 mx-auto mb-6 text-sm italic rounded-xl border shadow-sm text-stone-400 bg-surface border-stone-200 max-w-[700px]">"No climate data available. Configure data sources in Settings."</div> }.into_any()
    } else {
        let readings = StoredValue::new(readings);
        view! {
            <div>
                {move || {
                    let u = unit.get();
                    readings.get_value().iter().map(|r| {
                        let (temp_val, temp_unit_str) = if u == "F" {
                            let f = (r.temperature * 9.0 / 5.0) + 32.0;
                            (format!("{:.1}", f), "F")
                        } else {
                            (format!("{:.1}", r.temperature), "C")
                        };

                        let name = r.zone_name.clone();
                        let humidity = r.humidity;
                        let vpd = r.vpd;
                        let ago = format_time_ago(&r.recorded_at);

                        view! {
                            <div class="p-4 mx-auto mb-6 rounded-xl border shadow-sm bg-surface border-stone-200 max-w-[700px] dark:border-stone-700">
                                <div class="flex flex-wrap gap-4 justify-between items-center">
                                    <h3 class="m-0 text-base text-stone-700 dark:text-stone-300">{name}</h3>
                                    <div class="flex flex-wrap gap-6 items-center">
                                        <div class="flex flex-col items-center">
                                            <span class="text-xs font-medium tracking-wider uppercase text-stone-400">"Temp"</span>
                                            <span class="text-xl font-semibold text-primary">{temp_val}" "{temp_unit_str}</span>
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
            </div>
        }.into_any()
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
