use leptos::prelude::*;
use crate::app::ClimateData;

#[component]
pub fn ClimateDashboard(data: StoredValue<Vec<ClimateData>>, unit: Memo<String>) -> impl IntoView {
    let cd = data.get_value();
    if cd.is_empty() {
        view! { <div class="flex justify-center p-4 mx-auto mb-6 text-sm italic rounded-xl border shadow-sm text-stone-400 bg-surface border-stone-200 max-w-[700px]">"No climate data available (Configure AC Infinity Action)"</div> }.into_any()
    } else {
        view! {
            <div>
                {move || {
                    let u = unit.get();
                    data.get_value().iter().map(|dev| {
                        let (temp_val, temp_unit_str) = if u == "F" {
                            let f = (dev.temperature * 9.0 / 5.0) + 32.0;
                            (format!("{:.1}", f), "F")
                        } else {
                            (format!("{:.1}", dev.temperature), "C")
                        };

                        let name = dev.name.clone();
                        let humidity = dev.humidity;
                        let vpd = dev.vpd;
                        let updated = dev.updated.clone();

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
                                            <span class="text-xl font-semibold text-primary">{humidity}"%"</span>
                                        </div>
                                        <div class="flex flex-col items-center">
                                            <span class="text-xs font-medium tracking-wider uppercase text-stone-400">"VPD"</span>
                                            <span class="text-xl font-semibold text-primary">{vpd}" kPa"</span>
                                        </div>
                                    </div>
                                </div>
                                <div class="mt-2 text-xs text-right text-stone-400">
                                    "Updated: " {updated}
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()
                }}
            </div>
        }.into_any()
    }
}
