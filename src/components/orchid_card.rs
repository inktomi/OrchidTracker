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

    let suggestion_class = if is_misplaced { "text-red-600 font-bold" } else { "text-green-700" };

    let conservation = orchid.conservation_status.clone();

    view! {
        <div class="border border-gray-300 rounded-lg p-4 bg-white shadow hover:-translate-y-0.5 hover:shadow-md transition-transform cursor-pointer">
            <div on:click=move |_| on_select(orchid_clone.clone())>
                <h3 class="mt-0 text-primary">{orchid.name}</h3>
                <p><strong>"Species: "</strong> {orchid.species}</p>

                {conservation.map(|status| {
                    view! { <p class="italic text-red-700 my-1"><strong>"Status: "</strong> {status}</p> }
                })}

                <p><strong>"Watering: "</strong> "Every " {orchid.water_frequency_days} " days"</p>
                <p><strong>"Light Req: "</strong> {orchid.light_requirement.to_string()} " (" {orchid.light_lux} " Lux)"</p>
                <p><strong>"Temp Range: "</strong> {orchid.temperature_range}</p>
                <p><strong>"Placement: "</strong> {orchid.placement.to_string()}</p>
                <p class=suggestion_class><strong>"Suggestion: "</strong> {suggestion_msg}</p>
                <p><strong>"Notes: "</strong> {orchid.notes}</p>
            </div>
            <button class="bg-danger text-white border-none p-2 mt-4 text-sm rounded cursor-pointer hover:bg-danger-dark" on:click=move |ev: web_sys::MouseEvent| {
                ev.stop_propagation();
                on_delete(orchid_id);
            }>"Delete"</button>
        </div>
    }
}
