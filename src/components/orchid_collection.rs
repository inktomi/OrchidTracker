use leptos::prelude::*;
use crate::components::cabinet_table::OrchidCabinetTable;
use crate::components::orchid_card::OrchidCard;
use crate::model::ViewMode;
use crate::orchid::{Orchid, GrowingZone};

const TAB_ACTIVE: &str = "flex gap-1.5 items-center py-2 px-4 text-sm font-semibold rounded-lg border-none shadow-sm transition-all cursor-pointer text-primary bg-surface dark:text-primary-light";
const TAB_INACTIVE: &str = "flex gap-1.5 items-center py-2 px-4 text-sm font-medium bg-transparent rounded-lg border-none transition-all cursor-pointer text-stone-500 hover:text-stone-700 dark:text-stone-400 dark:hover:text-stone-200";

#[component]
pub fn OrchidCollection(
    orchids: Memo<Vec<Orchid>>,
    zones: Memo<Vec<GrowingZone>>,
    view_mode: Memo<ViewMode>,
    on_set_view: impl Fn(ViewMode) + 'static + Copy + Send + Sync,
    on_delete: impl Fn(String) + 'static + Copy + Send + Sync,
    on_select: impl Fn(Orchid) + 'static + Copy + Send + Sync,
    on_update: impl Fn(Orchid) + 'static + Copy + Send + Sync,
    on_water: impl Fn(String) + 'static + Copy + Send + Sync,
    on_add: impl Fn() + 'static + Copy + Send + Sync,
    on_scan: impl Fn() + 'static + Copy + Send + Sync,
    #[prop(optional)] read_only: bool,
) -> impl IntoView {
    let (sort_needs_water, set_sort_needs_water) = signal(false);
    let is_empty = Memo::new(move |_| orchids.get().is_empty());

    view! {
        <Show
            when=move || !is_empty.get()
            fallback=move || {
                if read_only {
                    view! { <p class="py-12 text-center text-stone-400">"This collection is empty."</p> }.into_any()
                } else {
                    view! { <EmptyCollection on_add=on_add on_scan=on_scan /> }.into_any()
                }
            }
        >
            // View toggle
            <div class="flex justify-center mb-6">
                <div class="inline-flex gap-1 p-1 rounded-xl bg-secondary">
                    <button
                        class=move || if view_mode.get() == ViewMode::Grid && !sort_needs_water.get() { TAB_ACTIVE } else { TAB_INACTIVE }
                        on:click=move |_| { set_sort_needs_water.set(false); on_set_view(ViewMode::Grid); }
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                            <path d="M5 3a2 2 0 00-2 2v2a2 2 0 002 2h2a2 2 0 002-2V5a2 2 0 00-2-2H5zM5 11a2 2 0 00-2 2v2a2 2 0 002 2h2a2 2 0 002-2v-2a2 2 0 00-2-2H5zM11 5a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V5zM11 13a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"/>
                        </svg>
                        "All Plants"
                    </button>
                    <button
                        class=move || if sort_needs_water.get() { TAB_ACTIVE } else { TAB_INACTIVE }
                        on:click=move |_| { set_sort_needs_water.set(true); on_set_view(ViewMode::Grid); }
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                            <path fill-rule="evenodd" d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z" clip-rule="evenodd"/>
                        </svg>
                        "Needs Water"
                    </button>
                    <button
                        class=move || if view_mode.get() == ViewMode::Table { TAB_ACTIVE } else { TAB_INACTIVE }
                        on:click=move |_| { set_sort_needs_water.set(false); on_set_view(ViewMode::Table); }
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                            <path d="M2 4.5A2.5 2.5 0 014.5 2h11A2.5 2.5 0 0118 4.5v2A2.5 2.5 0 0115.5 9h-11A2.5 2.5 0 012 6.5v-2zM2 13.5A2.5 2.5 0 014.5 11h11a2.5 2.5 0 012.5 2.5v2a2.5 2.5 0 01-2.5 2.5h-11A2.5 2.5 0 012 15.5v-2z"/>
                        </svg>
                        "By Zone"
                    </button>
                </div>
            </div>

            // Current view — reactive closure only depends on view_mode,
            // so watering (which changes orchids data, not view_mode) does NOT
            // recreate the grid. The <For> inside OrchidGrid handles that.
            {move || {
                match view_mode.get() {
                    ViewMode::Grid => view! {
                        <OrchidGrid
                            orchids=orchids
                            zones=zones
                            sort_needs_water=sort_needs_water
                            on_delete=on_delete
                            on_select=on_select
                            on_water=on_water
                            read_only=read_only
                        />
                    }.into_any(),
                    ViewMode::Table => {
                        let orchids_for_table = orchids.get();
                        let zones_for_table = zones.get();
                        view! {
                            <OrchidCabinetTable
                                orchids=orchids_for_table
                                zones=zones_for_table
                                on_delete=on_delete
                                on_select=on_select
                                on_update=on_update
                            />
                        }.into_any()
                    }
                }
            }}
        </Show>
    }.into_any()
}

/// Grid view with a stable `<For>` — orchid cards update in place when data
/// changes, preserving scroll position. Uses a composite key that includes
/// `last_watered_at` so only the watered card is replaced by `<For>`.
#[component]
fn OrchidGrid(
    orchids: Memo<Vec<Orchid>>,
    zones: Memo<Vec<GrowingZone>>,
    sort_needs_water: ReadSignal<bool>,
    on_delete: impl Fn(String) + 'static + Copy + Send + Sync,
    on_select: impl Fn(Orchid) + 'static + Copy + Send + Sync,
    on_water: impl Fn(String) + 'static + Copy + Send + Sync,
    read_only: bool,
) -> impl IntoView {
    view! {
        <div class="grid gap-5 grid-cols-[repeat(auto-fill,minmax(300px,1fr))]">
            <For
                each=move || {
                    let mut list = orchids.get();
                    if sort_needs_water.get() {
                        list.sort_by(|a, b| {
                            let a_due = a.days_until_due().unwrap_or(i64::MAX);
                            let b_due = b.days_until_due().unwrap_or(i64::MAX);
                            a_due.cmp(&b_due)
                        });
                    }
                    list
                }
                key=|orchid| format!(
                    "{}-{}",
                    orchid.id,
                    orchid.last_watered_at.map(|d| d.timestamp_millis()).unwrap_or(0)
                )
                children=move |orchid| {
                    let zones_clone = zones.get();
                    view! {
                        <OrchidCard
                            orchid=orchid
                            zones=zones_clone
                            on_delete=on_delete
                            on_select=on_select
                            on_water=on_water
                            read_only=read_only
                        />
                    }
                }
            />
        </div>
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
            // Decorative empty state image
            <div class="flex relative justify-center items-center mb-8 w-64 h-64 sm:w-72 sm:h-72">
                <div class="w-full h-full empty-plant [&>svg]:w-full [&>svg]:h-full" inner_html=include_str!("../../public/svg/empty_collection.svg")></div>
            </div>

            <h2 class="mb-3 text-2xl sm:text-3xl text-stone-800 dark:text-stone-100">"Your collection awaits"</h2>
            <p class="mb-8 max-w-sm text-sm leading-relaxed text-stone-400 dark:text-stone-500">
                "Your growing zones are all set up. Now add your first plant \u{2014} "
                "enter it by hand, or snap a photo of a plant tag and let AI look it up for you."
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
                    "Add Your First Plant"
                </button>
                <button
                    class="flex gap-2 justify-center items-center py-3 px-6 text-sm font-medium rounded-xl border transition-all duration-200 cursor-pointer text-stone-600 bg-surface border-stone-200/60 dark:text-stone-300 dark:bg-stone-800 dark:border-stone-700 dark:hover:border-primary-light/30 dark:hover:bg-primary-light/5 hover:border-primary/30 hover:bg-primary/5 active:scale-[0.98]"
                    on:click=move |_| on_scan()
                >
                    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M17.707 9.293a1 1 0 010 1.414l-7 7a1 1 0 01-1.414 0l-7-7A.997.997 0 012 10V5a3 3 0 013-3h5c.256 0 .512.098.707.293l7 7zM5 6a1 1 0 100-2 1 1 0 000 2z" clip-rule="evenodd"/>
                    </svg>
                    "ID a Plant"
                </button>
            </div>

            // Subtle hint
            <p class="mt-6 text-xs text-stone-300 dark:text-stone-600">
                "Tip: Scan a tag with your camera or search by name \u{2014} AI identifies species and checks zone compatibility"
            </p>
        </div>
    }.into_any()
}
