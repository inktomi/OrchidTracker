use leptos::prelude::*;
use crate::app::ClimateData;

#[component]
pub fn ClimateDashboard(data: StoredValue<Vec<ClimateData>>, unit: ReadSignal<String>) -> impl IntoView {
    let cd = data.get_value();
    if cd.is_empty() {
        view! { <div class="climate-dashboard empty">"No climate data available (Configure AC Infinity Action)"</div> }.into_any()
    } else {
        view! {
            <div class="climate-dashboard-container">
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
                            <div class="climate-dashboard">
                                <h3>{name}</h3>
                                <div class="climate-stat">
                                    <span class="label">"Temperature"</span>
                                    <span class="value">{temp_val} " " {temp_unit_str}</span>
                                </div>
                                <div class="climate-stat">
                                    <span class="label">"Humidity"</span>
                                    <span class="value">{humidity} "%"</span>
                                </div>
                                <div class="climate-stat">
                                    <span class="label">"VPD"</span>
                                    <span class="value">{vpd} " kPa"</span>
                                </div>
                                <div class="climate-footer">
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
