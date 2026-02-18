use leptos::prelude::*;
use super::BTN_GHOST;

#[component]
pub fn AppHeader(
    dark_mode: Memo<bool>,
    on_toggle_dark: impl Fn() + 'static + Copy + Send + Sync,
    on_add: impl Fn() + 'static + Copy + Send + Sync,
    on_scan: impl Fn() + 'static + Copy + Send + Sync,
    on_settings: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    view! {
        <header class="bg-primary">
            <div class="flex flex-wrap gap-3 justify-between items-center py-3 px-4 mx-auto sm:px-6 max-w-[1200px]">
                <h1 class="m-0 text-xl tracking-wide text-white">"Orchid Tracker"</h1>
                <div class="flex flex-wrap gap-2 items-center">
                    <button class=BTN_GHOST on:click=move |_| on_toggle_dark()>
                        {move || if dark_mode.get() { "\u{2600}" } else { "\u{263E}" }}
                    </button>
                    <button class=BTN_GHOST on:click=move |_| on_add()>"Add"</button>
                    <button class=BTN_GHOST on:click=move |_| on_scan()>"Scan"</button>
                    <button class=BTN_GHOST on:click=move |_| on_settings()>"Settings"</button>
                </div>
            </div>
        </header>
    }.into_any()
}
