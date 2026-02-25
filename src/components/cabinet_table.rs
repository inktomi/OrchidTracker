use super::BTN_DANGER;
use crate::orchid::{
    check_zone_compatibility, GrowingZone, Hemisphere, LightRequirement, LocationType, Orchid,
};
use crate::watering::ClimateSnapshot;
use leptos::prelude::*;

const SECTION_BASE: &str = "rounded-xl border p-4 bg-surface border-stone-200 shadow-sm transition-all dark:border-stone-700";
const SECTION_DRAG_OVER: &str = "ring-2 ring-primary-light/30 bg-primary-light/5";
const TH_CLASS: &str = "py-3 px-3 text-left text-xs font-semibold tracking-wider uppercase border-b text-stone-400 border-stone-200 bg-secondary dark:text-stone-500 dark:border-stone-700";
const TD_CLASS: &str =
    "py-3 px-3 text-left text-sm border-b border-stone-100 dark:border-stone-800";

fn border_color_for_light(light: &LightRequirement) -> &'static str {
    match light {
        LightRequirement::High => "border-t-shelf-high",
        LightRequirement::Medium => "border-t-shelf-medium",
        LightRequirement::Low => "border-t-shelf-low",
    }
}

#[component]
pub fn OrchidCabinetTable(
    orchids: Memo<Vec<Orchid>>,
    zones: Memo<Vec<GrowingZone>>,
    climate_snapshots: Option<Memo<Vec<ClimateSnapshot>>>,
    hemisphere: Option<Memo<String>>,
    on_delete: impl Fn(String) + 'static + Copy + Send + Sync,
    on_select: impl Fn(Orchid) + 'static + Copy + Send + Sync,
    on_update: impl Fn(Orchid) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let (drag_target, set_drag_target) = signal::<Option<String>>(None);

    let indoor_zones = Memo::new(move |_| {
        zones
            .get()
            .into_iter()
            .filter(|z| z.location_type == LocationType::Indoor)
            .collect::<Vec<_>>()
    });

    let outdoor_zones = Memo::new(move |_| {
        zones
            .get()
            .into_iter()
            .filter(|z| z.location_type == LocationType::Outdoor)
            .collect::<Vec<_>>()
    });

    let render_zone_section = move |zone: GrowingZone| {
        let zone_name = zone.name.clone();
        let zone_name_for_drop = zone_name.clone();
        let zone_name_for_dragover = zone_name.clone();
        let zone_name_for_check = zone_name.clone();
        let border = border_color_for_light(&zone.light_level);

        let zone_orchids = Memo::new({
            let zone_name = zone_name.clone();
            move |_| {
                orchids
                    .get()
                    .into_iter()
                    .filter(|o| o.placement == zone_name)
                    .collect::<Vec<_>>()
            }
        });

        let handle_drop = move |ev: leptos::ev::DragEvent| {
            ev.prevent_default();
            set_drag_target.set(None);
            #[cfg(feature = "hydrate")]
            {
                if let Some(data) = ev.data_transfer() {
                    if let Ok(id_str) = data.get_data("text/plain") {
                        let new_placement = zone_name_for_drop.clone();
                        let current_orchids = orchids.get();
                        if let Some(mut orchid) =
                            current_orchids.iter().find(|o| o.id == id_str).cloned()
                        {
                            if orchid.placement != new_placement {
                                orchid.placement = new_placement;
                                on_update(orchid);
                            }
                        }
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
                <OrchidTableSection
                    orchids=zone_orchids
                    zones=zones
                    climate_snapshots=climate_snapshots
                    hemisphere=hemisphere
                    on_delete=on_delete
                    on_select=on_select
                />
            </div>
        }
    };

    view! {
        <div class="flex flex-col gap-8">
            <h2 class="m-0">"Growing Zones"</h2>

            <Show when=move || !indoor_zones.get().is_empty()>
                <h3 class="m-0 text-sm font-semibold tracking-wider uppercase text-stone-400">"Indoor"</h3>
            </Show>

            <For
                each=move || indoor_zones.get()
                key=|zone| zone.id.clone()
                children=move |zone| render_zone_section(zone)
            />

            <Show when=move || !outdoor_zones.get().is_empty()>
                <h3 class="m-0 text-sm font-semibold tracking-wider uppercase text-stone-400">"Outdoor"</h3>
            </Show>

            <For
                each=move || outdoor_zones.get()
                key=|zone| zone.id.clone()
                children=move |zone| render_zone_section(zone)
            />
        </div>
    }
}

#[component]
fn OrchidTableSection(
    orchids: Memo<Vec<Orchid>>,
    zones: Memo<Vec<GrowingZone>>,
    climate_snapshots: Option<Memo<Vec<ClimateSnapshot>>>,
    hemisphere: Option<Memo<String>>,
    on_delete: impl Fn(String) + 'static + Copy + Send + Sync,
    on_select: impl Fn(Orchid) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    view! {
        <Show
            when=move || !orchids.get().is_empty()
            fallback=|| view! { <p class="p-4 text-sm italic text-center text-stone-400">"No orchids in this zone."</p> }
        >
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
                            each=move || orchids.get()
                            key=|orchid| orchid.id.clone()
                            children=move |orchid| {
                                let orchid_id = orchid.id.clone();
                                let orchid_clone = orchid.clone();

                                let is_misplaced = Memo::new({
                                    let placement = orchid.placement.clone();
                                    let light = orchid.light_requirement.clone();
                                    move |_| !check_zone_compatibility(&placement, &light, &zones.get())
                                });

                                let status_class = move || {
                                    if is_misplaced.get() {
                                        format!("{} text-danger font-semibold", TD_CLASS)
                                    } else {
                                        format!("{} text-primary-light font-semibold", TD_CLASS)
                                    }
                                };

                                let status_text = move || if is_misplaced.get() { "Move Needed" } else { "OK" };

                                let watering_text = Memo::new({
                                    let orchid = orchid.clone();
                                    move |_| {
                                        let snaps = climate_snapshots.map(|m| m.get()).unwrap_or_default();
                                        let hemi_str = hemisphere.map(|m| m.get()).unwrap_or_else(|| "N".to_string());
                                        let hemi = Hemisphere::from_code(&hemi_str);

                                        let snap = snaps.iter().find(|s| s.zone_name == orchid.placement).cloned();
                                        let estimate = orchid.climate_adjusted_water_frequency(&hemi, snap.as_ref());

                                        if estimate.climate_active {
                                            format!("Every ~{} days", estimate.adjusted_days)
                                        } else {
                                            format!("Every {} days", orchid.water_frequency_days)
                                        }
                                    }
                                });

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
                                        <td class=TD_CLASS><span class="font-medium text-primary dark:text-primary-light">{orchid.name.clone()}</span></td>
                                        <td class=format!("{} italic", TD_CLASS)>{orchid.species.clone()}</td>
                                        <td class=TD_CLASS>{move || watering_text.get()}</td>
                                        <td class=TD_CLASS>{orchid.light_requirement.to_string()}</td>
                                        <td class=TD_CLASS>{orchid.temperature_range.clone()}</td>
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
        </Show>
    }
}
