use leptos::prelude::*;
use crate::orchid::{Orchid, Placement};

const SECTION_BASE: &str = "border border-gray-300 rounded-lg p-4 bg-white";
const TH_CLASS: &str = "py-3 px-3 text-left border-b border-gray-200 bg-secondary font-bold";
const TD_CLASS: &str = "py-3 px-3 text-left border-b border-gray-100";

#[component]
pub fn OrchidCabinetTable(
    orchids: Vec<Orchid>,
    on_delete: impl Fn(u64) + 'static + Copy + Send + Sync,
    on_select: impl Fn(Orchid) + 'static + Copy + Send + Sync,
    on_update: impl Fn(Orchid) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let high_orchids: Vec<Orchid> = orchids.iter().filter(|o| o.placement == Placement::High).cloned().collect();
    let medium_orchids: Vec<Orchid> = orchids.iter().filter(|o| o.placement == Placement::Medium).cloned().collect();
    let low_orchids: Vec<Orchid> = orchids.iter().filter(|o| o.placement == Placement::Low).cloned().collect();
    let patio_orchids: Vec<Orchid> = orchids.iter().filter(|o| o.placement == Placement::Patio).cloned().collect();
    let outdoor_orchids: Vec<Orchid> = orchids.iter().filter(|o| o.placement == Placement::OutdoorRack).cloned().collect();

    let handle_drop = move |ev: leptos::ev::DragEvent, new_placement: Placement| {
        ev.prevent_default();
        if let Some(data) = ev.data_transfer() {
            if let Ok(id_str) = data.get_data("text/plain") {
                if let Ok(id) = id_str.parse::<u64>() {
                     if let Some(mut orchid) = orchids.iter().find(|o| o.id == id).cloned() {
                         if orchid.placement != new_placement {
                             orchid.placement = new_placement;
                             on_update(orchid);
                         }
                     }
                }
            }
        }
    };

    view! {
        <div class="flex flex-col gap-8">
            <h2>"Orchidarium Layout"</h2>

            <div
                class=format!("{} border-t-4 border-t-shelf-high", SECTION_BASE)
                on:dragover=move |ev: leptos::ev::DragEvent| ev.prevent_default()
                on:drop={
                    let handle_drop = handle_drop.clone();
                    move |ev| handle_drop(ev, Placement::High)
                }
            >
                <h3 class="pb-2 mt-0 border-b-2 text-primary border-secondary">"Top Shelf (High Light - Near Lights)"</h3>
                <OrchidTableSection orchids=high_orchids on_delete=on_delete on_select=on_select />
            </div>

            <div
                class=format!("{} border-t-4 border-t-shelf-medium", SECTION_BASE)
                on:dragover=move |ev: leptos::ev::DragEvent| ev.prevent_default()
                on:drop={
                    let handle_drop = handle_drop.clone();
                    move |ev| handle_drop(ev, Placement::Medium)
                }
            >
                <h3 class="pb-2 mt-0 border-b-2 text-primary border-secondary">"Middle Shelf (Medium Light)"</h3>
                <OrchidTableSection orchids=medium_orchids on_delete=on_delete on_select=on_select />
            </div>

            <div
                class=format!("{} border-t-4 border-t-shelf-low", SECTION_BASE)
                on:dragover=move |ev: leptos::ev::DragEvent| ev.prevent_default()
                on:drop={
                    let handle_drop = handle_drop.clone();
                    move |ev| handle_drop(ev, Placement::Low)
                }
            >
                <h3 class="pb-2 mt-0 border-b-2 text-primary border-secondary">"Bottom Shelf (Low Light - Floor)"</h3>
                <OrchidTableSection orchids=low_orchids on_delete=on_delete on_select=on_select />
            </div>

            <h2>"Outdoors"</h2>

            <div
                class=format!("{} border-t-4 border-t-primary", SECTION_BASE)
                on:dragover=move |ev: leptos::ev::DragEvent| ev.prevent_default()
                on:drop={
                    let handle_drop = handle_drop.clone();
                    move |ev| handle_drop(ev, Placement::OutdoorRack)
                }
            >
                <h3 class="pb-2 mt-0 border-b-2 text-primary border-secondary">"Outdoor Rack (High Sun)"</h3>
                <OrchidTableSection orchids=outdoor_orchids on_delete=on_delete on_select=on_select />
            </div>

            <div
                class=format!("{} border-t-4 border-t-primary", SECTION_BASE)
                on:dragover=move |ev: leptos::ev::DragEvent| ev.prevent_default()
                on:drop={
                    let handle_drop = handle_drop.clone();
                    move |ev| handle_drop(ev, Placement::Patio)
                }
            >
                <h3 class="pb-2 mt-0 border-b-2 text-primary border-secondary">"Patio (Morning Sun / Afternoon Shade)"</h3>
                <OrchidTableSection orchids=patio_orchids on_delete=on_delete on_select=on_select />
            </div>
        </div>
    }
}

#[component]
fn OrchidTableSection(
    orchids: Vec<Orchid>,
    on_delete: impl Fn(u64) + 'static + Copy + Send + Sync,
    on_select: impl Fn(Orchid) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    if orchids.is_empty() {
        view! { <p class="p-4 italic text-center text-gray-500">"No orchids on this shelf."</p> }.into_any()
    } else {
        view! {
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
                        key=|orchid| orchid.id
                        children=move |orchid| {
                            let orchid_id = orchid.id;
                            let orchid_clone = orchid.clone();
                            let is_misplaced = !orchid.placement.is_compatible_with(&orchid.light_requirement);
                            let status_class = if is_misplaced {
                                format!("{} text-danger font-bold", TD_CLASS)
                            } else {
                                format!("{} text-primary font-bold", TD_CLASS)
                            };
                            let status_text = if is_misplaced { "Move Needed" } else { "OK" };

                            view! {
                                <tr
                                    class="cursor-pointer hover:bg-gray-100"
                                    draggable="true"
                                    on:click=move |_| on_select(orchid_clone.clone())
                                    on:dragstart=move |ev: leptos::ev::DragEvent| {
                                        if let Some(data) = ev.data_transfer() {
                                            let _ = data.set_data("text/plain", &orchid_id.to_string());
                                        }
                                    }
                                >
                                    <td class=TD_CLASS>{orchid.name}</td>
                                    <td class=TD_CLASS>{orchid.species}</td>
                                    <td class=TD_CLASS>"Every " {orchid.water_frequency_days} " days"</td>
                                    <td class=TD_CLASS>{orchid.light_requirement.to_string()}</td>
                                    <td class=TD_CLASS>{orchid.temperature_range}</td>
                                    <td class=status_class>{status_text}</td>
                                    <td class=TD_CLASS>
                                        <button class="py-1 px-2 text-sm text-white bg-red-300 rounded border-none cursor-pointer hover:bg-red-500" on:click=move |ev: web_sys::MouseEvent| {
                                            ev.stop_propagation();
                                            on_delete(orchid_id);
                                        }>"X"</button>
                                    </td>
                                </tr>
                            }
                        }
                    />
                </tbody>
            </table>
        }.into_any()
    }
}
