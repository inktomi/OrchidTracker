use leptos::prelude::*;
use crate::orchid::{Orchid, GrowingZone, check_zone_compatibility};
use super::BTN_DANGER;

const BTN_WATER: &str = "flex gap-1 items-center py-1.5 px-3 text-xs font-semibold rounded-lg border-none cursor-pointer transition-colors text-sky-700 bg-sky-100 hover:bg-sky-200 dark:text-sky-300 dark:bg-sky-900/30 dark:hover:bg-sky-900/50";

#[component]
pub fn OrchidCard(
    orchid: Orchid,
    zones: Vec<GrowingZone>,
    on_delete: impl Fn(String) + 'static + Copy + Send + Sync,
    on_select: impl Fn(Orchid) + 'static + Copy + Send + Sync,
    on_water: impl Fn(String) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let orchid_id = orchid.id.clone();
    let orchid_id_water = orchid.id.clone();
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
    let has_first_bloom = orchid.first_bloom_at.is_some();
    let has_notes = !orchid.notes.is_empty();
    let notes = orchid.notes.clone();

    // Watering status
    let watering_text = match orchid.days_until_due() {
        Some(days) if days < 0 => format!("Overdue by {} days", -days),
        Some(0) => "Due today".to_string(),
        Some(1) => "Due tomorrow".to_string(),
        Some(days) if days <= 2 => format!("Due in {} days", days),
        _ => match orchid.days_since_watered() {
            Some(0) => "Watered today".to_string(),
            Some(1) => "Watered 1d ago".to_string(),
            Some(d) => format!("Watered {}d ago", d),
            None => format!("Every {} days", orchid.water_frequency_days),
        },
    };
    let is_overdue = orchid.is_overdue();
    let watering_class = if is_overdue {
        "font-medium text-danger"
    } else {
        "font-medium text-stone-700 dark:text-stone-300"
    };

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

                {has_first_bloom.then(|| {
                    view! { <span class="inline-block py-0.5 px-2 mb-3 ml-1 text-xs font-medium text-amber-700 rounded-full border dark:text-amber-300 bg-amber-100/80 border-amber-300/40 dark:bg-amber-900/30 dark:border-amber-700/40">"\u{1F33C} First Bloom!"</span> }
                })}

                <div class="grid grid-cols-2 gap-y-3 gap-x-4 text-sm">
                    <div>
                        <div class="text-xs tracking-wide text-stone-400">"Water"</div>
                        <div class=watering_class>{watering_text}</div>
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
            <div class="flex gap-2 justify-end py-3 px-5 border-t border-stone-100 dark:border-stone-800">
                <button class=BTN_WATER on:click={
                    let id = orchid_id_water.clone();
                    move |ev: leptos::ev::MouseEvent| {
                        ev.stop_propagation();
                        on_water(id.clone());
                    }
                }>
                    // Droplet SVG icon
                    <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z" clip-rule="evenodd"/>
                    </svg>
                    "Water"
                </button>
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
