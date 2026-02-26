use crate::orchid::{Hemisphere, Orchid};
use crate::watering::ClimateSnapshot;
use leptos::prelude::*;

#[component]
pub fn TodayTasks(
    orchids: Memo<Vec<Orchid>>,
    climate_snapshots: Memo<Vec<ClimateSnapshot>>,
    hemisphere: Memo<String>,
    on_select: impl Fn(Orchid) + 'static + Copy + Send + Sync,
    on_water: impl Fn(String) + 'static + Copy + Send + Sync,
    on_water_all: impl Fn(Vec<String>) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    // Determine which orchids are due for watering today
    let tasks_data = Memo::new(move |_| {
        let current_hemisphere = Hemisphere::from_code(&hemisphere.get());
        let snapshots = climate_snapshots.get();
        let current_orchids = orchids.get();

        let mut due_orchids = Vec::new();

        for orchid in current_orchids {
            let zone_snapshot = snapshots.iter().find(|s| s.zone_name == orchid.placement);
            let days_until = orchid.climate_days_until_due(&current_hemisphere, zone_snapshot);

            // If days_until is <= 0 or None (never watered), they need watering today.
            let needs_water = days_until.map(|d| d <= 0).unwrap_or(true);

            if needs_water {
                due_orchids.push((orchid, days_until));
            }
        }

        // Sort: most overdue first, then by name
        due_orchids.sort_by(|a, b| {
            let a_due = a.1.unwrap_or(-999);
            let b_due = b.1.unwrap_or(-999);
            a_due.cmp(&b_due).then(a.0.name.cmp(&b.0.name))
        });

        due_orchids
    });

    let due_count = Memo::new(move |_| tasks_data.get().len());

    let handle_water_all = move |_| {
        let ids: Vec<String> = tasks_data.get().into_iter().map(|(o, _)| o.id).collect();
        if !ids.is_empty() {
            on_water_all(ids);
        }
    };

    view! {
        <div class="flex flex-col gap-6 duration-500 animate-in fade-in slide-in-from-bottom-4 fill-mode-both">
            // Header Section with Glassmorphic Hero
            <div class="overflow-hidden relative p-6 rounded-3xl border shadow-sm sm:p-8 backdrop-blur-md bg-stone-50/80 border-stone-200/50 dark:bg-stone-900/60 dark:border-stone-700/50">
                // Decorative organic background shapes
                <div class="absolute top-0 right-0 w-64 h-64 rounded-full translate-x-1/3 -translate-y-1/2 pointer-events-none bg-primary/10 blur-3xl dark:bg-primary-light/10"></div>
                <div class="absolute bottom-0 left-0 w-48 h-48 rounded-full -translate-x-1/4 translate-y-1/3 pointer-events-none bg-sky-500/10 blur-2xl dark:bg-sky-400/10"></div>

                <div class="flex relative z-10 flex-col gap-4 justify-between items-start sm:flex-row sm:items-end">
                    <div>
                        <h2 class="font-serif text-3xl text-stone-800 drop-shadow-sm dark:text-stone-100">"Today's Tasks"</h2>
                        <p class="mt-2 max-w-md leading-relaxed text-stone-600 dark:text-stone-400">
                            {move || match due_count.get() {
                                0 => "All your plants are hydrated and happy. Enjoy the peace of your greenhouse.".to_string(),
                                1 => "Just one orchid needs your attention today.".to_string(),
                                n => format!("{} orchids are waiting for a drink today.", n),
                            }}
                        </p>
                    </div>

                    {move || if due_count.get() > 0 {
                        view! {
                            <button
                                class="flex overflow-hidden relative gap-2 items-center py-3 px-6 text-sm font-semibold text-white rounded-full shadow-md transition-all duration-300 hover:shadow-lg hover:-translate-y-0.5 focus:ring-2 focus:ring-offset-2 focus:outline-none group bg-primary dark:focus:ring-offset-stone-900 hover:bg-primary-light focus:ring-primary"
                                on:click=handle_water_all
                            >
                                // Shimmer effect
                                <div class="absolute inset-0 bg-gradient-to-r from-transparent to-transparent -translate-x-full via-white/20 group-hover:animate-shimmer"></div>

                                <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 transition-transform duration-300 group-hover:scale-110" viewBox="0 0 20 20" fill="currentColor">
                                    <path fill-rule="evenodd" d="M10 2a1 1 0 00-1 1v1a1 1 0 002 0V3a1 1 0 00-1-1zM4 4h3a3 3 0 006 0h3a2 2 0 012 2v9a2 2 0 01-2 2H4a2 2 0 01-2-2V6a2 2 0 012-2zm2.5 7a1.5 1.5 0 100-3 1.5 1.5 0 000 3zm2.45 4a2.5 2.5 0 10-4.9 0h4.9zM12 9a1 1 0 100 2h3a1 1 0 100-2h-3zm-1 4a1 1 0 011-1h2a1 1 0 110 2h-2a1 1 0 01-1-1z" clip-rule="evenodd" />
                                </svg>
                                "Water All Due"
                            </button>
                        }.into_any()
                    } else {
                        view! { <div/> }.into_any()
                    }}
                </div>
            </div>

            // Task List
            {move || {
                let tasks = tasks_data.get();
                if tasks.is_empty() {
                    view! {
                        <div class="flex flex-col justify-center items-center py-16 px-4 text-center rounded-3xl border border-dashed border-stone-200 dark:border-stone-700/50">
                            <div class="mb-6 w-24 h-24 opacity-80 text-stone-300 botanical-sway dark:text-stone-700/50">
                                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor">
                                    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8z"/>
                                    <path d="M12.5 7H11v6l5.25 3.15.75-1.23-4.5-2.67z"/>
                                </svg>
                            </div>
                            <h3 class="text-xl font-medium text-stone-700 dark:text-stone-300">"All Caught Up"</h3>
                            <p class="mt-2 text-stone-500 dark:text-stone-400">"Your orchids are thriving. Check back tomorrow!"</p>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                            {tasks.into_iter().enumerate().map(|(i, (orchid, days_until))| {
                                let orchid_clone = orchid.clone();
                                let orchid_id = orchid.id.clone();

                                let status_text = match days_until {
                                    None => "Needs first watering".to_string(),
                                    Some(0) => "Due today".to_string(),
                                    Some(1) => "Due tomorrow".to_string(),
                                    Some(d) if d < 0 => format!("{} days overdue", -d),
                                    Some(d) => format!("Due in {} days", d),
                                };

                                let status_color = match days_until {
                                    None | Some(0) => "text-amber-600 bg-amber-50 dark:text-amber-400 dark:bg-amber-900/20",
                                    Some(d) if d < 0 => "text-danger bg-danger/10 dark:text-red-400 dark:bg-red-900/20",
                                    _ => "text-sky-600 bg-sky-50 dark:text-sky-400 dark:bg-sky-900/20",
                                };

                                // Staggered animation delay
                                let delay_class = format!("animation-delay-{}", (i % 5) * 100);

                                view! {
                                    <div
                                        class=format!("group flex relative flex-col p-5 bg-white rounded-2xl border shadow-sm transition-all duration-300 cursor-pointer dark:bg-stone-800 border-stone-100 dark:border-stone-700 hover:shadow-md hover:border-primary/30 dark:hover:border-primary-light/30 animate-in fade-in slide-in-from-bottom-2 fill-mode-both {}", delay_class)
                                        on:click=move |_| on_select(orchid_clone.clone())
                                    >
                                        <div class="flex justify-between items-start mb-3">
                                            <div class="flex flex-col min-w-0">
                                                <h4 class="font-serif text-lg transition-colors truncate text-stone-800 dark:text-stone-100 dark:group-hover:text-primary-light group-hover:text-primary">
                                                    {orchid.name.clone()}
                                                </h4>
                                                <p class="text-sm italic truncate text-stone-500 dark:text-stone-400">
                                                    {orchid.species.clone()}
                                                </p>
                                            </div>
                                            <button
                                                class="flex flex-shrink-0 justify-center items-center w-10 h-10 rounded-full transition-colors text-sky-600 bg-sky-50 dark:bg-sky-900/30 dark:text-sky-400 dark:hover:bg-sky-900/50 hover:bg-sky-100 hover:text-sky-700"
                                                on:click=move |e| {
                                                    e.prevent_default();
                                                    e.stop_propagation();
                                                    on_water(orchid_id.clone());
                                                }
                                                aria-label=format!("Water {}", orchid.name)
                                                title="Mark as watered"
                                            >
                                                <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 20 20" fill="currentColor">
                                                    <path fill-rule="evenodd" d="M3.172 5.172a4 4 0 015.656 0L10 6.343l1.172-1.171a4 4 0 115.656 5.656L10 17.657l-6.828-6.829a4 4 0 010-5.656z" clip-rule="evenodd" />
                                                </svg>
                                            </button>
                                        </div>

                                        <div class="mt-auto">
                                            <div class="flex gap-2 items-center">
                                                <span class=format!("px-2.5 py-1 text-xs font-semibold rounded-md {}", status_color)>
                                                    {status_text}
                                                </span>
                                                <span class="flex gap-1 items-center text-xs text-stone-400 dark:text-stone-500">
                                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 20 20" fill="currentColor">
                                                        <path fill-rule="evenodd" d="M5.05 4.05a7 7 0 119.9 9.9L10 18.9l-4.95-4.95a7 7 0 010-9.9zM10 11a2 2 0 100-4 2 2 0 000 4z" clip-rule="evenodd" />
                                                    </svg>
                                                    {orchid.placement.clone()}
                                                </span>
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }
            }}

            // CSS for shimmer and animation delays
            <style>
                "
                @keyframes shimmer {
                    100% {
                        transform: translateX(100%);
                    }
                }
                .animate-shimmer {
                    animation: shimmer 2s infinite;
                }
                .animation-delay-0 { animation-delay: 0ms; }
                .animation-delay-100 { animation-delay: 100ms; }
                .animation-delay-200 { animation-delay: 200ms; }
                .animation-delay-300 { animation-delay: 300ms; }
                .animation-delay-400 { animation-delay: 400ms; }
                "
            </style>
        </div>
    }
}
