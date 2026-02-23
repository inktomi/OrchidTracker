use leptos::prelude::*;
use crate::orchid::{HabitatWeather, HabitatWeatherSummary, ClimateReading};

const CARD: &str = "p-4 mt-4 rounded-xl border shadow-sm bg-gradient-to-br from-emerald-50/50 to-stone-50 border-emerald-200/60 dark:from-emerald-950/20 dark:to-stone-900 dark:border-emerald-800/40";
const STAT_LABEL: &str = "text-xs font-medium tracking-wider uppercase text-stone-400";
const STAT_VALUE: &str = "text-lg font-semibold text-emerald-700 dark:text-emerald-400";

#[component]
pub fn HabitatWeatherCard(
    native_region: String,
    latitude: f64,
    longitude: f64,
    zone_reading: Option<ClimateReading>,
) -> impl IntoView {
    let lat = latitude;
    let lon = longitude;

    let habitat_resource = Resource::new(
        move || (lat, lon),
        move |(lat, lon)| crate::server_fns::climate::get_habitat_current(lat, lon),
    );

    let history_resource = Resource::new(
        move || (lat, lon),
        move |(lat, lon)| crate::server_fns::climate::get_habitat_history(lat, lon, 30),
    );

    let region = native_region.clone();

    view! {
        <div class=CARD>
            <div class="flex gap-2 justify-between items-start mb-3">
                <div>
                    <h4 class="m-0 text-sm font-semibold text-emerald-800 dark:text-emerald-300">"Native Habitat Weather"</h4>
                    <p class="mt-0.5 mb-0 text-xs text-stone-500 dark:text-stone-400">{region}</p>
                </div>
                <span class="py-0.5 px-2 text-xs font-medium text-emerald-700 bg-emerald-100 rounded-full dark:text-emerald-400 dark:bg-emerald-900/40">
                    {format!("{:.1}, {:.1}", latitude, longitude)}
                </span>
            </div>

            <Suspense fallback=move || view! { <p class="text-xs text-stone-400">"Loading habitat weather..."</p> }>
                {move || {
                    let weather = habitat_resource.get().and_then(|r| r.ok()).flatten();
                    let zone = zone_reading.clone();

                    match weather {
                        Some(hw) => {
                            view! { <HabitatCurrentView weather=hw zone_reading=zone /> }.into_any()
                        }
                        None => {
                            view! {
                                <p class="text-xs italic text-stone-400">"No habitat weather data yet. Data will appear after the next poll cycle."</p>
                            }.into_any()
                        }
                    }
                }}
            </Suspense>

            <Suspense fallback=|| ()>
                {move || {
                    let summaries = history_resource.get()
                        .and_then(|r| r.ok())
                        .unwrap_or_default();

                    if summaries.is_empty() {
                        None
                    } else {
                        Some(view! { <HabitatTrendView summaries=summaries /> })
                    }
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn HabitatCurrentView(
    weather: HabitatWeather,
    zone_reading: Option<ClimateReading>,
) -> impl IntoView {
    let ago = format_time_ago(&weather.recorded_at);

    view! {
        <div class="flex flex-wrap gap-4 items-center">
            <div class="flex flex-col items-center">
                <span class=STAT_LABEL>"Temp"</span>
                <span class=STAT_VALUE>{format!("{:.1}", weather.temperature)}" C"</span>
            </div>
            <div class="flex flex-col items-center">
                <span class=STAT_LABEL>"Humidity"</span>
                <span class=STAT_VALUE>{format!("{:.0}", weather.humidity)}"%"</span>
            </div>
            <div class="flex flex-col items-center">
                <span class=STAT_LABEL>"Precip"</span>
                <span class=STAT_VALUE>{format!("{:.1}", weather.precipitation)}" mm"</span>
            </div>

            {zone_reading.map(|zr| {
                let temp_diff = weather.temperature - zr.temperature;
                let diff_sign = if temp_diff >= 0.0 { "+" } else { "" };
                let diff_color = if temp_diff.abs() < 3.0 {
                    "text-emerald-600 dark:text-emerald-400"
                } else if temp_diff.abs() < 6.0 {
                    "text-amber-600 dark:text-amber-400"
                } else {
                    "text-red-600 dark:text-red-400"
                };
                view! {
                    <div class="flex flex-col items-center pl-4 ml-2 border-l border-stone-200 dark:border-stone-700">
                        <span class=STAT_LABEL>"vs Your Zone"</span>
                        <span class=format!("text-lg font-semibold {}", diff_color)>
                            {format!("{}{:.1} C", diff_sign, temp_diff)}
                        </span>
                        <span class="text-xs text-stone-400">{zr.zone_name.clone()}</span>
                    </div>
                }
            })}
        </div>
        <div class="mt-2 text-xs text-right text-stone-400">"Updated: " {ago}</div>
    }.into_any()
}

#[component]
fn HabitatTrendView(summaries: Vec<HabitatWeatherSummary>) -> impl IntoView {
    let recent: Vec<_> = summaries.into_iter().take(7).collect();

    view! {
        <div class="pt-3 mt-3 border-t border-emerald-200/40 dark:border-emerald-800/30">
            <h5 class="mt-0 mb-2 text-xs font-semibold tracking-wider uppercase text-stone-500 dark:text-stone-400">"Recent Trends"</h5>
            <div class="flex gap-1 items-end h-12">
                {recent.iter().map(|s| {
                    // Normalize temp to bar height (0-48px range, centered around 20C)
                    let normalized = ((s.avg_temperature - 10.0) / 30.0).clamp(0.1, 1.0);
                    let height = format!("{}px", (normalized * 48.0) as u32);
                    let title = format!(
                        "{}: {:.1}C ({:.1}-{:.1}), {:.0}% humidity",
                        s.period_type, s.avg_temperature, s.min_temperature, s.max_temperature, s.avg_humidity
                    );
                    view! {
                        <div
                            class="flex-1 rounded-sm bg-emerald-400/60 dark:bg-emerald-600/40"
                            style=format!("height: {}", height)
                            title=title
                        ></div>
                    }
                }).collect::<Vec<_>>()}
            </div>
            {recent.first().map(|s| {
                view! {
                    <p class="mt-1 mb-0 text-xs text-stone-400">
                        "Avg: " {format!("{:.1}C", s.avg_temperature)}
                        " / Range: " {format!("{:.1}-{:.1}C", s.min_temperature, s.max_temperature)}
                        " / Humidity: " {format!("{:.0}%", s.avg_humidity)}
                    </p>
                }
            })}
        </div>
    }.into_any()
}

fn format_time_ago(dt: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let diff = now - *dt;

    if diff.num_minutes() < 1 {
        "just now".to_string()
    } else if diff.num_minutes() < 60 {
        format!("{}m ago", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{}h ago", diff.num_hours())
    } else {
        format!("{}d ago", diff.num_days())
    }
}
