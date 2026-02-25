use leptos::prelude::*;
use crate::orchid::{Orchid, GrowingZone, LightRequirement, LocationType, Hemisphere, check_zone_compatibility};
use crate::watering::ClimateSnapshot;
use super::BTN_DANGER;

const SECTION_BASE: &str = "rounded-xl border p-4 bg-surface border-stone-200 shadow-sm transition-all dark:border-stone-700";
const SECTION_DRAG_OVER: &str = "ring-2 ring-primary-light/30 bg-primary-light/5";
const TH_CLASS: &str = "py-3 px-3 text-left text-xs font-semibold tracking-wider uppercase border-b text-stone-400 border-stone-200 bg-secondary dark:text-stone-500 dark:border-stone-700";
const TD_CLASS: &str = "py-3 px-3 text-left text-sm border-b border-stone-100 dark:border-stone-800";

fn border_color_for_light(light: &LightRequirement) -> &'static str {
    match light {
        LightRequirement::High => "border-t-shelf-high",
        LightRequirement::Medium => "border-t-shelf-medium",
        LightRequirement::Low => "border-t-shelf-low",
    }
}

#[component]
pub fn OrchidCabinetTable(
    orchids: Vec<Orchid>,
    zones: Vec<GrowingZone>,
    #[prop(default = Vec::new())] climate_snapshots: Vec<ClimateSnapshot>,
    #[prop(default = String::new())] hemisphere: String,
    on_delete: impl Fn(String) + 'static + Copy + Send + Sync,
    on_select: impl Fn(Orchid) + 'static + Copy + Send + Sync,
    on_update: impl Fn(Orchid) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let (drag_target, set_drag_target) = signal::<Option<String>>(None);

    let indoor_zones: Vec<GrowingZone> = zones.iter().filter(|z| z.location_type == LocationType::Indoor).cloned().collect();
    let outdoor_zones: Vec<GrowingZone> = zones.iter().filter(|z| z.location_type == LocationType::Outdoor).cloned().collect();

    let orchids_stored = StoredValue::new(orchids);
    let zones_stored = StoredValue::new(zones);
    let snapshots_stored = StoredValue::new(climate_snapshots);
    let hemi = Hemisphere::from_code(&hemisphere);

    let render_zone_section = move |zone: GrowingZone| {
        let zone_name = zone.name.clone();
        let zone_name_for_drop = zone_name.clone();
        let zone_name_for_dragover = zone_name.clone();
        let zone_name_for_check = zone_name.clone();
        let border = border_color_for_light(&zone.light_level);
        let zone_orchids: Vec<Orchid> = orchids_stored.with_value(|orchids| {
            orchids.iter().filter(|o| o.placement == zone_name).cloned().collect()
        });
        let zones_for_section = zones_stored.with_value(|z| z.clone());

        let handle_drop = move |ev: leptos::ev::DragEvent| {
            ev.prevent_default();
            set_drag_target.set(None);
            #[cfg(feature = "hydrate")]
            {
                if let Some(data) = ev.data_transfer() {
                    if let Ok(id_str) = data.get_data("text/plain") {
                        let new_placement = zone_name_for_drop.clone();
                        orchids_stored.with_value(|orchids| {
                            if let Some(mut orchid) = orchids.iter().find(|o| o.id == id_str).cloned() {
                                if orchid.placement != new_placement {
                                    orchid.placement = new_placement;
                                    on_update(orchid);
                                }
                            }
                        });
                    }
                }
            }
            #[cfg(not(feature = "hydrate"))]
            {
                let _ = (&on_update, &zone_name_for_drop);
            }
        };

        let display_name = format!("{} ({} Light)", zone.name, zone.light_level);

        view! {
            <div
                class=move || {
                    let base = format!("{} border-t-4 {}", SECTION_BASE, border);
                    if drag_target.get().as_deref() == Some(&zone_name_for_check) {
                        format!("{} {}", base, SECTION_DRAG_OVER)
                    } else { base }
                }
                on:dragover={
                    let name = zone_name_for_dragover.clone();
                    move |ev: leptos::ev::DragEvent| {
                        ev.prevent_default();
                        set_drag_target.set(Some(name.clone()));
                    }
                }
                on:dragleave=move |_| set_drag_target.set(None)
                on:drop=handle_drop
            >
                <h3 class="pb-2 mt-0 border-b text-primary border-stone-200 dark:border-stone-700">{display_name}</h3>
                <OrchidTableSection orchids=zone_orchids zones=zones_for_section climate_snapshots=snapshots_stored.get_value() hemisphere=hemi.clone() on_delete=on_delete on_select=on_select />
            </div>
        }
    };

    view! {
        <div class="flex flex-col gap-8">
            <h2 class="m-0">"Growing Zones"</h2>

            {if !indoor_zones.is_empty() {
                Some(view! {
                    <h3 class="m-0 text-sm font-semibold tracking-wider uppercase text-stone-400">"Indoor"</h3>
                })
            } else {
                None
            }}

            {indoor_zones.into_iter().map(|zone| {
                render_zone_section(zone)
            }).collect::<Vec<_>>()}

            {if !outdoor_zones.is_empty() {
                Some(view! {
                    <h3 class="m-0 text-sm font-semibold tracking-wider uppercase text-stone-400">"Outdoor"</h3>
                })
            } else {
                None
            }}

            {outdoor_zones.into_iter().map(|zone| {
                render_zone_section(zone)
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
fn OrchidTableSection(
    orchids: Vec<Orchid>,
    zones: Vec<GrowingZone>,
    climate_snapshots: Vec<ClimateSnapshot>,
    hemisphere: Hemisphere,
    on_delete: impl Fn(String) + 'static + Copy + Send + Sync,
    on_select: impl Fn(Orchid) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    if orchids.is_empty() {
        view! { <p class="p-4 text-sm italic text-center text-stone-400">"No orchids in this zone."</p> }.into_any()
    } else {
        view! {
            <div class="overflow-x-auto">
                <table class="mt-4 w-full border-collapse">
                    <thead>
                        <tr>
                            <th class=TH_CLASS>"Name"</th>
                            <th class=TH_CLASS>"Species"</th>
                            <th class=TH_CLASS>"Watering"</th>
                            <th class=TH_CLASS>"Light Req"</th>
                            <th class=TH_CLASS>"Temp Range"</th>
                            <th class=TH_CLASS>"Status"</th>
                            <th class=TH_CLASS>"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For
                            each=move || orchids.clone()
                            key=|orchid| orchid.id.clone()
                            children=move |orchid| {
                                let orchid_id = orchid.id.clone();
                                let orchid_clone = orchid.clone();
                                let is_misplaced = !check_zone_compatibility(&orchid.placement, &orchid.light_requirement, &zones);
                                let status_class = if is_misplaced {
                                    format!("{} text-danger font-semibold", TD_CLASS)
                                } else {
                                    format!("{} text-primary-light font-semibold", TD_CLASS)
                                };
                                let status_text = if is_misplaced { "Move Needed" } else { "OK" };

                                let snap = climate_snapshots.iter().find(|s| s.zone_name == orchid.placement).cloned();
                                let estimate = orchid.climate_adjusted_water_frequency(&hemisphere, snap.as_ref());
                                let watering_text = if estimate.climate_active {
                                    format!("Every ~{} days", estimate.adjusted_days)
                                } else {
                                    format!("Every {} days", orchid.water_frequency_days)
                                };

                                view! {
                                    <tr
                                        class="transition-colors cursor-pointer dark:hover:bg-stone-800/50 hover:bg-secondary/50"
                                        draggable="true"
                                        on:click=move |_| on_select(orchid_clone.clone())
                                        on:dragstart={
                                            let id = orchid_id.clone();
                                            move |ev: leptos::ev::DragEvent| {
                                                #[cfg(feature = "hydrate")]
                                                {
                                                    if let Some(data) = ev.data_transfer() {
                                                        let _ = data.set_data("text/plain", &id);
                                                    }
                                                }
                                                #[cfg(not(feature = "hydrate"))]
                                                {
                                                    let _ = (&ev, &id);
                                                }
                                            }
                                        }
                                    >
                                        <td class=TD_CLASS><span class="font-medium text-primary dark:text-primary-light">{orchid.name}</span></td>
                                        <td class=format!("{} italic", TD_CLASS)>{orchid.species}</td>
                                        <td class=TD_CLASS>{watering_text}</td>
                                        <td class=TD_CLASS>{orchid.light_requirement.to_string()}</td>
                                        <td class=TD_CLASS>{orchid.temperature_range}</td>
                                        <td class=status_class>{status_text}</td>
                                        <td class=TD_CLASS>
                                            <button class=BTN_DANGER on:click={
                                                let id = orchid.id.clone();
                                                move |ev: leptos::ev::MouseEvent| {
                                                    ev.stop_propagation();
                                                    on_delete(id.clone());
                                                }
                                            }>"Delete"</button>
                                        </td>
                                    </tr>
                                }
                            }
                        />
                    </tbody>
                </table>
            </div>
        }.into_any()
    }
}
