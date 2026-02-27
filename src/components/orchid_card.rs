use super::BTN_DANGER;
use crate::orchid::{check_zone_compatibility, GrowingZone, Hemisphere, Orchid};
use crate::watering::ClimateSnapshot;
use leptos::prelude::*;

const BTN_WATER: &str = "flex gap-1 items-center py-1.5 px-3 text-xs font-semibold rounded-lg border-none cursor-pointer transition-colors text-sky-700 bg-sky-100 hover:bg-sky-200 dark:text-sky-300 dark:bg-sky-900/30 dark:hover:bg-sky-900/50";

#[component]
pub fn OrchidCard(
    orchid: Orchid,
    zones: Vec<GrowingZone>,
    #[prop(default = None)] climate_snapshot: Option<ClimateSnapshot>,
    #[prop(default = String::new())] hemisphere: String,
    on_delete: impl Fn(String) + 'static + Copy + Send + Sync,
    on_select: impl Fn(Orchid) + 'static + Copy + Send + Sync,
    on_water: impl Fn(String) + 'static + Copy + Send + Sync,
    #[prop(optional)] read_only: bool,
) -> impl IntoView {
    let orchid_id = orchid.id.clone();
    let orchid_id_water = orchid.id.clone();
    let orchid_clone = orchid.clone();
    let is_misplaced =
        !check_zone_compatibility(&orchid.placement, &orchid.light_requirement, &zones);
    let mismatch_reason = if is_misplaced {
        let zone_light = zones
            .iter()
            .find(|z| z.name == orchid.placement)
            .map(|z| z.light_level.as_str())
            .unwrap_or("Unknown");
        Some(format!(
            "{} light zone \u{2014} needs {}",
            zone_light,
            orchid.light_requirement.as_str()
        ))
    } else {
        None
    };

    let conservation = orchid.conservation_status.clone();
    let has_first_bloom = orchid.first_bloom_at.is_some();
    let has_notes = !orchid.notes.is_empty();
    let notes = orchid.notes.clone();

    // Watering status — climate-aware when snapshot available
    let hemi = Hemisphere::from_code(&hemisphere);
    let estimate = orchid.climate_adjusted_water_frequency(&hemi, climate_snapshot.as_ref());
    let climate_active = estimate.climate_active;
    let approx = if climate_active { "~" } else { "" };

    let watering_text = match orchid.climate_days_until_due(&hemi, climate_snapshot.as_ref()) {
        Some(days) if days < 0 => format!("Overdue by {}{} days", approx, -days),
        Some(0) => "Due today".to_string(),
        Some(1) => "Due tomorrow".to_string(),
        Some(days) if days <= 2 => format!("Due in {}{} days", approx, days),
        _ => match orchid.days_since_watered() {
            Some(0) => "Watered today".to_string(),
            Some(1) => "Watered 1d ago".to_string(),
            Some(d) => format!("Watered {}d ago", d),
            None => {
                if climate_active {
                    format!("Every ~{} days", estimate.adjusted_days)
                } else {
                    format!("Every {} days", orchid.water_frequency_days)
                }
            }
        },
    };
    let is_overdue = orchid.is_climate_overdue(&hemi, climate_snapshot.as_ref());
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
                    {is_misplaced.then(|| view! {
                        <div class="flex-shrink-0 w-5 h-5 [&>svg]:w-full [&>svg]:h-full" inner_html=include_str!("../../public/svg/alert_warning_24.svg")></div>
                    })}
                </div>
                <p class="mt-0 mb-3 text-sm italic text-stone-500 dark:text-stone-400">{orchid.species}</p>

                <div class="flex flex-wrap gap-1 justify-between items-center mb-3">
                    <div class="flex flex-wrap gap-1">
                        {conservation.map(|status| {
                            view! { <span class="inline-block py-0.5 px-2 text-xs font-medium rounded-full border text-danger bg-danger/5 border-danger/20">{status}</span> }
                        })}
                        {has_first_bloom.then(|| {
                            view! { <span class="inline-block py-0.5 px-2 text-xs font-medium text-amber-700 rounded-full border dark:text-amber-300 bg-amber-100/80 border-amber-300/40 dark:bg-amber-900/30 dark:border-amber-700/40">"\u{1F33C} First Bloom!"</span> }
                        })}
                    </div>
                    {mismatch_reason.map(|reason| {
                        view! { <span class="text-xs text-amber-600 dark:text-amber-400">{reason}</span> }
                    })}
                </div>

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
                        <div class="text-xs tracking-wide text-stone-400">"Pot"</div>
                        <div class="font-medium text-stone-700 dark:text-stone-300">{orchid.pot_medium.clone().map(|v| v.to_string()).unwrap_or_else(|| "\u{2014}".to_string())}</div>
                    </div>
                </div>

                {has_notes.then(|| {
                    view! { <p class="mt-3 text-sm leading-relaxed text-stone-500 line-clamp-2">{notes.clone()}</p> }
                })}
            </div>
            {(!read_only).then(|| view! {
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
            })}
        </div>
    }
}

// ── SSR Component Rendering Tests ───────────────────────────────────

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use crate::test_helpers::{test_orchid, test_orchid_with_care};
    use leptos::reactive::owner::Owner;

    fn noop_string(_: String) {}
    fn noop_orchid(_: Orchid) {}

    #[test]
    fn test_orchid_card_renders_name_and_species() {
        let owner = Owner::new();
        owner.with(|| {
            let orchid = test_orchid();
            let html = view! {
                <OrchidCard
                    orchid=orchid
                    zones=vec![]
                    on_delete=noop_string
                    on_select=noop_orchid
                    on_water=noop_string
                />
            }
            .to_html();
            assert!(html.contains("Test Orchid"), "Should render orchid name");
            assert!(
                html.contains("Phalaenopsis"),
                "Should render orchid species"
            );
        });
    }

    #[test]
    fn test_orchid_card_hides_actions_when_read_only() {
        let owner = Owner::new();
        owner.with(|| {
            let orchid = test_orchid();
            let html = view! {
                <OrchidCard
                    orchid=orchid
                    zones=vec![]
                    on_delete=noop_string
                    on_select=noop_orchid
                    on_water=noop_string
                    read_only=true
                />
            }
            .to_html();
            assert!(
                !html.contains("Delete"),
                "Delete button should be hidden in read-only mode, got: {html}"
            );
            // "Water" appears as a stat label; check that the action bar is absent
            assert!(
                !html.contains("border-t border-stone-100"),
                "Action bar should be hidden in read-only mode"
            );
        });
    }

    #[test]
    fn test_orchid_card_shows_actions_when_not_read_only() {
        let owner = Owner::new();
        owner.with(|| {
            let orchid = test_orchid();
            let html = view! {
                <OrchidCard
                    orchid=orchid
                    zones=vec![]
                    on_delete=noop_string
                    on_select=noop_orchid
                    on_water=noop_string
                    read_only=false
                />
            }
            .to_html();
            assert!(
                html.contains("Delete"),
                "Delete button should be visible when read_only=false"
            );
            assert!(
                html.contains("Water"),
                "Water button should be visible when read_only=false"
            );
        });
    }

    #[test]
    fn test_orchid_card_shows_watering_status() {
        let owner = Owner::new();
        owner.with(|| {
            let orchid = test_orchid();
            let html = view! {
                <OrchidCard
                    orchid=orchid
                    zones=vec![]
                    on_delete=noop_string
                    on_select=noop_orchid
                    on_water=noop_string
                />
            }
            .to_html();
            // Never-watered orchid shows "Every N days"
            assert!(
                html.contains("Every 7 days"),
                "Should show watering frequency for never-watered orchid"
            );
        });
    }

    #[test]
    fn test_orchid_card_shows_pot_medium() {
        let owner = Owner::new();
        owner.with(|| {
            let orchid = test_orchid_with_care();
            let html = view! {
                <OrchidCard
                    orchid=orchid
                    zones=vec![]
                    on_delete=noop_string
                    on_select=noop_orchid
                    on_water=noop_string
                />
            }
            .to_html();
            assert!(
                html.contains("Bark"),
                "Should show pot medium from care data"
            );
        });
    }
}
