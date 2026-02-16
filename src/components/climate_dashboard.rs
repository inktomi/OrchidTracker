use leptos::prelude::*;
use crate::app::ClimateData;

#[component]
pub fn ClimateDashboard(data: StoredValue<Vec<ClimateData>>, unit: Memo<String>) -> impl IntoView {
    let cd = data.get_value();
    if cd.is_empty() {
        view! { <div class="bg-white p-4 mb-4 rounded-lg shadow border border-gray-300 max-w-[600px] mx-auto flex justify-center italic text-gray-500">"No climate data available (Configure AC Infinity Action)"</div> }.into_any()
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
                            <div class="bg-white p-4 mb-4 rounded-lg shadow border border-gray-300 max-w-[600px] mx-auto flex justify-around flex-wrap text-gray-800">
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
                                <div class="w-full text-center text-[0.7rem] text-gray-400 mt-2">
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
