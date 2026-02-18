use leptos::prelude::*;
use crate::orchid::{Orchid, Placement};
use super::BTN_DANGER;

const SECTION_BASE: &str = "rounded-xl border p-4 bg-surface border-stone-200 shadow-sm transition-all dark:border-stone-700";
const SECTION_DRAG_OVER: &str = "ring-2 ring-primary-light/30 bg-primary-light/5";
const TH_CLASS: &str = "py-3 px-3 text-left text-xs font-semibold tracking-wider uppercase border-b text-stone-400 border-stone-200 bg-secondary dark:text-stone-500 dark:border-stone-700";
const TD_CLASS: &str = "py-3 px-3 text-left text-sm border-b border-stone-100 dark:border-stone-800";

#[component]
pub fn OrchidCabinetTable(
    orchids: Vec<Orchid>,
    on_delete: impl Fn(String) + 'static + Copy + Send + Sync,
    on_select: impl Fn(Orchid) + 'static + Copy + Send + Sync,
    on_update: impl Fn(Orchid) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let high_orchids: Vec<Orchid> = orchids.iter().filter(|o| o.placement == Placement::High).cloned().collect();
    let medium_orchids: Vec<Orchid> = orchids.iter().filter(|o| o.placement == Placement::Medium).cloned().collect();
    let low_orchids: Vec<Orchid> = orchids.iter().filter(|o| o.placement == Placement::Low).cloned().collect();
    let patio_orchids: Vec<Orchid> = orchids.iter().filter(|o| o.placement == Placement::Patio).cloned().collect();
    let outdoor_orchids: Vec<Orchid> = orchids.iter().filter(|o| o.placement == Placement::OutdoorRack).cloned().collect();

    let (drag_target, set_drag_target) = signal::<Option<Placement>>(None);

    let handle_drop = move |ev: leptos::ev::DragEvent, new_placement: Placement| {
        ev.prevent_default();
        set_drag_target.set(None);
        #[cfg(feature = "hydrate")]
        {
            if let Some(data) = ev.data_transfer() {
                if let Ok(id_str) = data.get_data("text/plain") {
                    if let Some(mut orchid) = orchids.iter().find(|o| o.id == id_str).cloned() {
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
            let _ = (&on_update, &new_placement);
        }
    };

    view! {
        <div class="flex flex-col gap-8">
            <h2 class="m-0">"Orchidarium Layout"</h2>

            <div
                class=move || {
                    let base = format!("{} border-t-4 border-t-shelf-high", SECTION_BASE);
                    if drag_target.get() == Some(Placement::High) {
                        format!("{} {}", base, SECTION_DRAG_OVER)
                    } else { base }
                }
                on:dragover=move |ev: leptos::ev::DragEvent| {
                    ev.prevent_default();
                    set_drag_target.set(Some(Placement::High));
                }
                on:dragleave=move |_| set_drag_target.set(None)
                on:drop={
                    let handle_drop = handle_drop.clone();
                    move |ev| handle_drop(ev, Placement::High)
                }
            >
                <h3 class="pb-2 mt-0 border-b text-primary border-stone-200 dark:border-stone-700">"Top Shelf (High Light)"</h3>
                <OrchidTableSection orchids=high_orchids on_delete=on_delete on_select=on_select />
            </div>

            <div
                class=move || {
                    let base = format!("{} border-t-4 border-t-shelf-medium", SECTION_BASE);
                    if drag_target.get() == Some(Placement::Medium) {
                        format!("{} {}", base, SECTION_DRAG_OVER)
                    } else { base }
                }
                on:dragover=move |ev: leptos::ev::DragEvent| {
                    ev.prevent_default();
                    set_drag_target.set(Some(Placement::Medium));
                }
                on:dragleave=move |_| set_drag_target.set(None)
                on:drop={
                    let handle_drop = handle_drop.clone();
                    move |ev| handle_drop(ev, Placement::Medium)
                }
            >
                <h3 class="pb-2 mt-0 border-b text-primary border-stone-200 dark:border-stone-700">"Middle Shelf (Medium Light)"</h3>
                <OrchidTableSection orchids=medium_orchids on_delete=on_delete on_select=on_select />
            </div>

            <div
                class=move || {
                    let base = format!("{} border-t-4 border-t-shelf-low", SECTION_BASE);
                    if drag_target.get() == Some(Placement::Low) {
                        format!("{} {}", base, SECTION_DRAG_OVER)
                    } else { base }
                }
                on:dragover=move |ev: leptos::ev::DragEvent| {
                    ev.prevent_default();
                    set_drag_target.set(Some(Placement::Low));
                }
                on:dragleave=move |_| set_drag_target.set(None)
                on:drop={
                    let handle_drop = handle_drop.clone();
                    move |ev| handle_drop(ev, Placement::Low)
                }
            >
                <h3 class="pb-2 mt-0 border-b text-primary border-stone-200 dark:border-stone-700">"Bottom Shelf (Low Light)"</h3>
                <OrchidTableSection orchids=low_orchids on_delete=on_delete on_select=on_select />
            </div>

            <h2 class="m-0">"Outdoors"</h2>

            <div
                class=move || {
                    let base = format!("{} border-t-4 border-t-primary-light", SECTION_BASE);
                    if drag_target.get() == Some(Placement::OutdoorRack) {
                        format!("{} {}", base, SECTION_DRAG_OVER)
                    } else { base }
                }
                on:dragover=move |ev: leptos::ev::DragEvent| {
                    ev.prevent_default();
                    set_drag_target.set(Some(Placement::OutdoorRack));
                }
                on:dragleave=move |_| set_drag_target.set(None)
                on:drop={
                    let handle_drop = handle_drop.clone();
                    move |ev| handle_drop(ev, Placement::OutdoorRack)
                }
            >
                <h3 class="pb-2 mt-0 border-b text-primary border-stone-200 dark:border-stone-700">"Outdoor Rack (High Sun)"</h3>
                <OrchidTableSection orchids=outdoor_orchids on_delete=on_delete on_select=on_select />
            </div>

            <div
                class=move || {
                    let base = format!("{} border-t-4 border-t-primary-light", SECTION_BASE);
                    if drag_target.get() == Some(Placement::Patio) {
                        format!("{} {}", base, SECTION_DRAG_OVER)
                    } else { base }
                }
                on:dragover=move |ev: leptos::ev::DragEvent| {
                    ev.prevent_default();
                    set_drag_target.set(Some(Placement::Patio));
                }
                on:dragleave=move |_| set_drag_target.set(None)
                on:drop={
                    let handle_drop = handle_drop.clone();
                    move |ev| handle_drop(ev, Placement::Patio)
                }
            >
                <h3 class="pb-2 mt-0 border-b text-primary border-stone-200 dark:border-stone-700">"Patio (Morning Sun / Afternoon Shade)"</h3>
                <OrchidTableSection orchids=patio_orchids on_delete=on_delete on_select=on_select />
            </div>
        </div>
    }
}

#[component]
fn OrchidTableSection(
    orchids: Vec<Orchid>,
    on_delete: impl Fn(String) + 'static + Copy + Send + Sync,
    on_select: impl Fn(Orchid) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    if orchids.is_empty() {
        view! { <p class="p-4 text-sm italic text-center text-stone-400">"No orchids on this shelf."</p> }.into_any()
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
                                let is_misplaced = !orchid.placement.is_compatible_with(&orchid.light_requirement);
                                let status_class = if is_misplaced {
                                    format!("{} text-danger font-semibold", TD_CLASS)
                                } else {
                                    format!("{} text-primary-light font-semibold", TD_CLASS)
                                };
                                let status_text = if is_misplaced { "Move Needed" } else { "OK" };

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
                                        <td class=TD_CLASS>"Every " {orchid.water_frequency_days} " days"</td>
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
