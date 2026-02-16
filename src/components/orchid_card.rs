use leptos::prelude::*;
use crate::orchid::Orchid;

#[component]
pub fn OrchidCard(
    orchid: Orchid,
    on_delete: impl Fn(u64) + 'static + Copy + Send + Sync,
    on_select: impl Fn(Orchid) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let orchid_id = orchid.id;
    let orchid_clone = orchid.clone();
    let is_misplaced = !orchid.placement.is_compatible_with(&orchid.light_requirement);
    let suggestion_msg = if is_misplaced {
        format!("(Needs {})", orchid.light_requirement)
    } else {
        " (Optimal)".to_string()
    };

    let suggestion_style = if is_misplaced { "color: red; font-weight: bold;" } else { "color: green;" };

    let conservation = orchid.conservation_status.clone();

    view! {
        <div class="orchid-card">
            <div class="card-content" on:click=move |_| on_select(orchid_clone.clone())>
                <h3>{orchid.name}</h3>
                <p><strong>"Species: "</strong> {orchid.species}</p>

                {conservation.map(|status| {
                    view! { <p class="conservation-status"><strong>"Status: "</strong> {status}</p> }
                })}

                <p><strong>"Watering: "</strong> "Every " {orchid.water_frequency_days} " days"</p>
                <p><strong>"Light Req: "</strong> {orchid.light_requirement.to_string()} " (" {orchid.light_lux} " Lux)"</p>
                <p><strong>"Temp Range: "</strong> {orchid.temperature_range}</p>
                <p><strong>"Placement: "</strong> {orchid.placement.to_string()}</p>
                <p style=suggestion_style><strong>"Suggestion: "</strong> {suggestion_msg}</p>
                <p><strong>"Notes: "</strong> {orchid.notes}</p>
            </div>
            <button class="delete-btn" on:click=move |ev: web_sys::MouseEvent| {
                ev.stop_propagation();
                on_delete(orchid_id);
            }>"Delete"</button>
        </div>
    }
}
