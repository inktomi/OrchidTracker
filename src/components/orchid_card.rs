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
        <div class="p-4 bg-white rounded-lg border border-gray-300 shadow transition-transform cursor-pointer hover:shadow-md hover:-translate-y-0.5">
            <div on:click=move |_| on_select(orchid_clone.clone())>
                <h3 class="mt-0 text-primary">{orchid.name}</h3>
                <p><strong>"Species: "</strong> {orchid.species}</p>

                {conservation.map(|status| {
                    view! { <p class="my-1 italic text-red-700"><strong>"Status: "</strong> {status}</p> }
                })}

                <p><strong>"Watering: "</strong> "Every " {orchid.water_frequency_days} " days"</p>
                <p><strong>"Light Req: "</strong> {orchid.light_requirement.to_string()} " (" {orchid.light_lux} " Lux)"</p>
                <p><strong>"Temp Range: "</strong> {orchid.temperature_range}</p>
                <p><strong>"Placement: "</strong> {orchid.placement.to_string()}</p>
                <p class=suggestion_class><strong>"Suggestion: "</strong> {suggestion_msg}</p>
                <p><strong>"Notes: "</strong> {orchid.notes}</p>
            </div>
            <button class="p-2 mt-4 text-sm text-white rounded border-none cursor-pointer bg-danger hover:bg-danger-dark" on:click=move |ev: web_sys::MouseEvent| {
                ev.stop_propagation();
                on_delete(orchid_id);
            }>"Delete"</button>
        </div>
    }
}
