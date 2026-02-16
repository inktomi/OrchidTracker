use leptos::prelude::*;
use crate::app::ClimateData;

#[component]
pub fn ClimateDashboard(data: StoredValue<Vec<ClimateData>>, unit: Memo<String>) -> impl IntoView {
    let cd = data.get_value();
    if cd.is_empty() {
        view! { <div class="flex justify-center p-4 mx-auto mb-4 italic text-gray-500 bg-white rounded-lg border border-gray-300 shadow max-w-[600px]">"No climate data available (Configure AC Infinity Action)"</div> }.into_any()
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
                            <div class="flex flex-wrap justify-around p-4 mx-auto mb-4 text-gray-800 bg-white rounded-lg border border-gray-300 shadow max-w-[600px]">
                                <h3>{name}</h3>
                                <div class="flex flex-col items-center p-2">
                                    <span class="text-xs text-gray-500 uppercase">"Temperature"</span>
                                    <span class="text-2xl font-bold text-primary">{temp_val} " " {temp_unit_str}</span>
                                </div>
                                <div class="flex flex-col items-center p-2">
                                    <span class="text-xs text-gray-500 uppercase">"Humidity"</span>
                                    <span class="text-2xl font-bold text-primary">{humidity} "%"</span>
                                </div>
                                <div class="flex flex-col items-center p-2">
                                    <span class="text-xs text-gray-500 uppercase">"VPD"</span>
                                    <span class="text-2xl font-bold text-primary">{vpd} " kPa"</span>
                                </div>
                                <div class="mt-2 w-full text-center text-gray-400 text-[0.7rem]">
                                    "Last Updated: " {updated}
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()
                }}
            </div>
        }.into_any()
    }
}
