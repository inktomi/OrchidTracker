use leptos::prelude::*;
use crate::orchid::{Orchid, Placement};

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
        <div class="cabinet-view">
            <h2>"Orchidarium Layout"</h2>

            <div
                class="cabinet-section high-section"
                on:dragover=move |ev: leptos::ev::DragEvent| ev.prevent_default()
                on:drop={
                    let handle_drop = handle_drop.clone();
                    move |ev| handle_drop(ev, Placement::High)
                }
            >
                <h3>"Top Shelf (High Light - Near Lights)"</h3>
                <OrchidTableSection orchids=high_orchids on_delete=on_delete on_select=on_select />
            </div>

            <div
                class="cabinet-section medium-section"
                on:dragover=move |ev: leptos::ev::DragEvent| ev.prevent_default()
                on:drop={
                    let handle_drop = handle_drop.clone();
                    move |ev| handle_drop(ev, Placement::Medium)
                }
            >
                <h3>"Middle Shelf (Medium Light)"</h3>
                <OrchidTableSection orchids=medium_orchids on_delete=on_delete on_select=on_select />
            </div>

            <div
                class="cabinet-section low-section"
                on:dragover=move |ev: leptos::ev::DragEvent| ev.prevent_default()
                on:drop={
                    let handle_drop = handle_drop.clone();
                    move |ev| handle_drop(ev, Placement::Low)
                }
            >
                <h3>"Bottom Shelf (Low Light - Floor)"</h3>
                <OrchidTableSection orchids=low_orchids on_delete=on_delete on_select=on_select />
            </div>

            <h2>"Outdoors"</h2>

            <div
                class="cabinet-section outdoor-section"
                on:dragover=move |ev: leptos::ev::DragEvent| ev.prevent_default()
                on:drop={
                    let handle_drop = handle_drop.clone();
                    move |ev| handle_drop(ev, Placement::OutdoorRack)
                }
            >
                <h3>"Outdoor Rack (High Sun)"</h3>
                <OrchidTableSection orchids=outdoor_orchids on_delete=on_delete on_select=on_select />
            </div>

            <div
                class="cabinet-section patio-section"
                on:dragover=move |ev: leptos::ev::DragEvent| ev.prevent_default()
                on:drop={
                    let handle_drop = handle_drop.clone();
                    move |ev| handle_drop(ev, Placement::Patio)
                }
            >
                <h3>"Patio (Morning Sun / Afternoon Shade)"</h3>
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
        view! { <p class="empty-shelf">"No orchids on this shelf."</p> }.into_any()
    } else {
        view! {
            <table class="orchid-table">
                <thead>
                    <tr>
                        <th>"Name"</th>
                        <th>"Species"</th>
                        <th>"Watering"</th>
                        <th>"Light Req"</th>
                        <th>"Temp Range"</th>
                        <th>"Status"</th>
                        <th>"Actions"</th>
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
                            let status_class = if is_misplaced { "status-warning" } else { "status-ok" };
                            let status_text = if is_misplaced { "Move Needed" } else { "OK" };

                            view! {
                                <tr
                                    class="clickable-row"
                                    draggable="true"
                                    on:click=move |_| on_select(orchid_clone.clone())
                                    on:dragstart=move |ev: leptos::ev::DragEvent| {
                                        if let Some(data) = ev.data_transfer() {
                                            let _ = data.set_data("text/plain", &orchid_id.to_string());
                                        }
                                    }
                                >
                                    <td>{orchid.name}</td>
                                    <td>{orchid.species}</td>
                                    <td>"Every " {orchid.water_frequency_days} " days"</td>
                                    <td>{orchid.light_requirement.to_string()}</td>
                                    <td>{orchid.temperature_range}</td>
                                    <td class=status_class>{status_text}</td>
                                    <td>
                                        <button class="delete-btn-small" on:click=move |ev: web_sys::MouseEvent| {
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
