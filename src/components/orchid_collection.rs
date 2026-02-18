use leptos::prelude::*;
use crate::components::cabinet_table::OrchidCabinetTable;
use crate::components::orchid_card::OrchidCard;
use crate::model::ViewMode;
use crate::orchid::Orchid;

const TAB_ACTIVE: &str = "py-2 px-4 text-sm font-semibold text-primary bg-surface rounded-lg shadow-sm cursor-pointer border-none transition-all dark:text-primary-light";
const TAB_INACTIVE: &str = "py-2 px-4 text-sm font-medium text-stone-500 bg-transparent rounded-lg cursor-pointer border-none transition-all hover:text-stone-700 dark:text-stone-400 dark:hover:text-stone-200";

#[component]
pub fn OrchidCollection(
    orchids_resource: Resource<Result<Vec<Orchid>, ServerFnError>>,
    view_mode: Memo<ViewMode>,
    on_set_view: impl Fn(ViewMode) + 'static + Copy + Send + Sync,
    on_delete: impl Fn(String) + 'static + Copy + Send + Sync,
    on_select: impl Fn(Orchid) + 'static + Copy + Send + Sync,
    on_update: impl Fn(Orchid) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    view! {
        <div class="flex justify-center mb-6">
            <div class="inline-flex gap-1 p-1 rounded-xl bg-secondary">
                <button
                    class=move || if view_mode.get() == ViewMode::Grid { TAB_ACTIVE } else { TAB_INACTIVE }
                    on:click=move |_| on_set_view(ViewMode::Grid)
                >
                    "Grid View"
                </button>
                <button
                    class=move || if view_mode.get() == ViewMode::Table { TAB_ACTIVE } else { TAB_INACTIVE }
                    on:click=move |_| on_set_view(ViewMode::Table)
                >
                    "Shelf View"
                </button>
            </div>
        </div>

        <Suspense fallback=move || view! { <p class="text-center text-stone-500">"Loading orchids..."</p> }>
            {move || orchids_resource.get().map(|result| {
                let orchids = result.unwrap_or_default();
                let orchids_for_table = orchids.clone();
                match view_mode.get() {
                    ViewMode::Grid => view! {
                        <div class="grid gap-5 grid-cols-[repeat(auto-fill,minmax(300px,1fr))]">
                            <For
                                each=move || orchids.clone()
                                key=|orchid| orchid.id.clone()
                                children=move |orchid| {
                                    let orchid_clone = orchid.clone();
                                    view! {
                                        <OrchidCard
                                            orchid=orchid_clone
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
                            on_delete=on_delete
                            on_select=on_select
                            on_update=on_update
                        />
                    }.into_any()
                }
            })}
        </Suspense>
    }.into_any()
}
