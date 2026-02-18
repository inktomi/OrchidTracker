use leptos::prelude::*;
use crate::orchid::{Orchid, GrowingZone, check_zone_compatibility};
use super::BTN_DANGER;

#[component]
pub fn OrchidCard(
    orchid: Orchid,
    zones: Vec<GrowingZone>,
    on_delete: impl Fn(String) + 'static + Copy + Send + Sync,
    on_select: impl Fn(Orchid) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let orchid_id = orchid.id.clone();
    let orchid_clone = orchid.clone();
    let is_misplaced = !check_zone_compatibility(&orchid.placement, &orchid.light_requirement, &zones);
    let suggestion_msg = if is_misplaced {
        format!("Needs {}", orchid.light_requirement)
    } else {
        "Optimal".to_string()
    };

    let status_class = if is_misplaced {
        "inline-flex py-1 px-2.5 text-xs font-semibold rounded-full bg-danger/10 text-danger"
    } else {
        "inline-flex py-1 px-2.5 text-xs font-semibold rounded-full bg-primary-light/10 text-primary-light"
    };

    let conservation = orchid.conservation_status.clone();
    let has_notes = !orchid.notes.is_empty();
    let notes = orchid.notes.clone();

    view! {
        <div class="overflow-hidden rounded-xl border shadow-sm transition-all duration-200 hover:shadow-md hover:-translate-y-0.5 bg-surface border-stone-200/80 dark:border-stone-700 dark:hover:border-stone-600 hover:border-stone-300">
            <div class="p-5 cursor-pointer" on:click=move |_| on_select(orchid_clone.clone())>
                <div class="flex gap-2 justify-between items-start mb-1">
                    <h3 class="m-0 text-primary">{orchid.name}</h3>
                    <span class=status_class>{suggestion_msg}</span>
                </div>
                <p class="mt-0 mb-3 text-sm italic text-stone-400">{orchid.species}</p>

                {conservation.map(|status| {
                    view! { <span class="inline-block py-0.5 px-2 mb-3 text-xs font-medium rounded-full border text-danger bg-danger/5 border-danger/20">{status}</span> }
                })}

                <div class="grid grid-cols-2 gap-y-3 gap-x-4 text-sm">
                    <div>
                        <div class="text-xs tracking-wide text-stone-400">"Water"</div>
                        <div class="font-medium text-stone-700 dark:text-stone-300">"Every " {orchid.water_frequency_days} " days"</div>
                    </div>
                    <div>
                        <div class="text-xs tracking-wide text-stone-400">"Light"</div>
                        <div class="font-medium text-stone-700 dark:text-stone-300">{orchid.light_requirement.to_string()}</div>
                    </div>
                    <div>
                        <div class="text-xs tracking-wide text-stone-400">"Zone"</div>
                        <div class="font-medium text-stone-700 dark:text-stone-300">{orchid.placement.clone()}</div>
                    </div>
                    <div>
                        <div class="text-xs tracking-wide text-stone-400">"Temp"</div>
                        <div class="font-medium text-stone-700 dark:text-stone-300">{orchid.temperature_range}</div>
                    </div>
                </div>

                {has_notes.then(|| {
                    view! { <p class="mt-3 text-sm leading-relaxed text-stone-500 line-clamp-2">{notes.clone()}</p> }
                })}
            </div>
            <div class="flex justify-end py-3 px-5 border-t border-stone-100 dark:border-stone-800">
                <button class=BTN_DANGER on:click={
                    let id = orchid_id.clone();
                    move |ev: leptos::ev::MouseEvent| {
                        ev.stop_propagation();
                        on_delete(id.clone());
                    }
                }>"Delete"</button>
            </div>
        </div>
    }
}
