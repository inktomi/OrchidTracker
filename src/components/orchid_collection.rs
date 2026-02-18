use leptos::prelude::*;
use crate::components::cabinet_table::OrchidCabinetTable;
use crate::components::orchid_card::OrchidCard;
use crate::model::ViewMode;
use crate::orchid::{Orchid, GrowingZone};

const TAB_ACTIVE: &str = "flex gap-1.5 items-center py-2 px-4 text-sm font-semibold rounded-lg border-none shadow-sm transition-all cursor-pointer text-primary bg-surface dark:text-primary-light";
const TAB_INACTIVE: &str = "flex gap-1.5 items-center py-2 px-4 text-sm font-medium bg-transparent rounded-lg border-none transition-all cursor-pointer text-stone-500 hover:text-stone-700 dark:text-stone-400 dark:hover:text-stone-200";

#[component]
pub fn OrchidCollection(
    orchids_resource: Resource<Result<Vec<Orchid>, ServerFnError>>,
    zones: Memo<Vec<GrowingZone>>,
    view_mode: Memo<ViewMode>,
    on_set_view: impl Fn(ViewMode) + 'static + Copy + Send + Sync,
    on_delete: impl Fn(String) + 'static + Copy + Send + Sync,
    on_select: impl Fn(Orchid) + 'static + Copy + Send + Sync,
    on_update: impl Fn(Orchid) + 'static + Copy + Send + Sync,
    on_add: impl Fn() + 'static + Copy + Send + Sync,
    on_scan: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    view! {
        <Suspense fallback=move || view! { <p class="text-center text-stone-500">"Loading orchids..."</p> }>
            {move || orchids_resource.get().map(|result| {
                let orchids = result.unwrap_or_default();

                if orchids.is_empty() {
                    view! { <EmptyCollection on_add=on_add on_scan=on_scan /> }.into_any()
                } else {
                    let orchids_for_table = orchids.clone();
                    let current_zones = zones.get();
                    let zones_for_table = current_zones.clone();

                    view! {
                        // View toggle
                        <div class="flex justify-center mb-6">
                            <div class="inline-flex gap-1 p-1 rounded-xl bg-secondary">
                                <button
                                    class=move || if view_mode.get() == ViewMode::Grid { TAB_ACTIVE } else { TAB_INACTIVE }
                                    on:click=move |_| on_set_view(ViewMode::Grid)
                                >
                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                                        <path d="M5 3a2 2 0 00-2 2v2a2 2 0 002 2h2a2 2 0 002-2V5a2 2 0 00-2-2H5zM5 11a2 2 0 00-2 2v2a2 2 0 002 2h2a2 2 0 002-2v-2a2 2 0 00-2-2H5zM11 5a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V5zM11 13a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"/>
                                    </svg>
                                    "All Plants"
                                </button>
                                <button
                                    class=move || if view_mode.get() == ViewMode::Table { TAB_ACTIVE } else { TAB_INACTIVE }
                                    on:click=move |_| on_set_view(ViewMode::Table)
                                >
                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                                        <path d="M2 4.5A2.5 2.5 0 014.5 2h11A2.5 2.5 0 0118 4.5v2A2.5 2.5 0 0115.5 9h-11A2.5 2.5 0 012 6.5v-2zM2 13.5A2.5 2.5 0 014.5 11h11a2.5 2.5 0 012.5 2.5v2a2.5 2.5 0 01-2.5 2.5h-11A2.5 2.5 0 012 15.5v-2z"/>
                                    </svg>
                                    "By Zone"
                                </button>
                            </div>
                        </div>

                        // Current view
                        {match view_mode.get() {
                            ViewMode::Grid => view! {
                                <div class="grid gap-5 grid-cols-[repeat(auto-fill,minmax(300px,1fr))]">
                                    <For
                                        each=move || orchids.clone()
                                        key=|orchid| orchid.id.clone()
                                        children=move |orchid| {
                                            let orchid_clone = orchid.clone();
                                            let zones_clone = current_zones.clone();
                                            view! {
                                                <OrchidCard
                                                    orchid=orchid_clone
                                                    zones=zones_clone
                                                    on_delete=on_delete
                                                    on_select=on_select
                                                />
                                            }
                                        }
                                    />
                                </div>
                            }.into_any(),
                            ViewMode::Table => view! {
                                <OrchidCabinetTable
                                    orchids=orchids_for_table
                                    zones=zones_for_table
                                    on_delete=on_delete
                                    on_select=on_select
                                    on_update=on_update
                                />
                            }.into_any()
                        }}
                    }.into_any()
                }
            })}
        </Suspense>
    }.into_any()
}

/// Warm, inviting empty state shown when the collection has no orchids yet.
#[component]
fn EmptyCollection(
    on_add: impl Fn() + 'static + Copy + Send + Sync,
    on_scan: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center py-16 px-6 text-center animate-fade-in">
            // Decorative floating dots (leaves)
            <div class="flex relative justify-center items-center mb-8 w-40 h-40">
                // Outer ring
                <div class="absolute inset-0 rounded-full border-2 border-dashed empty-ring border-primary/20 dark:border-primary-light/15"></div>

                // Floating leaf dots
                <div class="absolute -top-1 left-8 w-2.5 h-2.5 rounded-full bg-primary/40 dark:bg-primary-light/30 empty-dot"></div>
                <div class="absolute -right-1 top-6 w-2 h-2 rounded-full bg-accent/50 empty-dot"></div>
                <div class="absolute -left-1 bottom-2 w-3 h-3 rounded-full bg-primary-light/30 empty-dot"></div>
                <div class="absolute -bottom-1 right-8 w-2 h-2 rounded-full bg-accent-light/40 empty-dot"></div>
                <div class="absolute left-0 top-1 w-1.5 h-1.5 rounded-full bg-shelf-medium/40 empty-dot"></div>

                // Central plant
                <div class="text-6xl empty-plant">"🌿"</div>
            </div>

            <h2 class="mb-3 text-2xl sm:text-3xl text-stone-800 dark:text-stone-100">"Your collection awaits"</h2>
            <p class="mb-8 max-w-sm text-sm leading-relaxed text-stone-400 dark:text-stone-500">
                "Your growing zones are all set up. Now add your first orchid \u{2014} "
                "enter it by hand or point your camera at one to have the AI identify it."
            </p>

            // CTAs
            <div class="flex flex-col gap-3 w-full max-w-xs sm:flex-row sm:justify-center sm:max-w-none">
                <button
                    class="flex gap-2 justify-center items-center py-3 px-6 text-sm font-semibold text-white rounded-xl border-none transition-all duration-200 cursor-pointer hover:shadow-lg bg-primary hover:bg-primary-dark hover:shadow-primary/20 active:scale-[0.98]"
                    on:click=move |_| on_add()
                >
                    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                        <path d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z"/>
                    </svg>
                    "Add Your First Orchid"
                </button>
                <button
                    class="flex gap-2 justify-center items-center py-3 px-6 text-sm font-medium rounded-xl border transition-all duration-200 cursor-pointer text-stone-600 bg-surface border-stone-200/60 dark:text-stone-300 dark:bg-stone-800 dark:border-stone-700 dark:hover:border-primary-light/30 dark:hover:bg-primary-light/5 hover:border-primary/30 hover:bg-primary/5 active:scale-[0.98]"
                    on:click=move |_| on_scan()
                >
                    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M4 5a2 2 0 00-2 2v8a2 2 0 002 2h12a2 2 0 002-2V7a2 2 0 00-2-2h-1.586a1 1 0 01-.707-.293l-1.121-1.121A2 2 0 0011.172 3H8.828a2 2 0 00-1.414.586L6.293 4.707A1 1 0 015.586 5H4zm6 9a3 3 0 100-6 3 3 0 000 6z" clip-rule="evenodd"/>
                    </svg>
                    "Scan with Camera"
                </button>
            </div>

            // Subtle hint
            <p class="mt-6 text-xs text-stone-300 dark:text-stone-600">
                "Tip: The scanner uses AI to identify species and check zone compatibility"
            </p>
        </div>
    }.into_any()
}
