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
        <header class="overflow-hidden relative bg-primary">
            // Subtle gradient glow — matches auth pages
            <div class="absolute inset-0 bg-gradient-to-r from-primary via-primary-dark to-primary"></div>
            <div class="absolute inset-0 opacity-40 auth-glow-green-alt"></div>
            <div class="absolute inset-0 opacity-30 auth-glow-gold-alt"></div>

            <div class="flex relative z-10 flex-wrap gap-3 justify-between items-center py-3 px-4 mx-auto sm:px-6 max-w-[1200px]">
                <div class="flex gap-2.5 items-center">
                    <div class="flex justify-center items-center w-8 h-8 text-sm rounded-lg border bg-white/10 border-white/20">"🌿"</div>
                    <span class="text-sm font-semibold tracking-widest uppercase text-white/90">"Orchid Tracker"</span>
                </div>
                <div class="flex flex-wrap gap-2 items-center">
                    <button class=BTN_GHOST on:click=move |_| on_toggle_dark()>
                        {move || if dark_mode.get() { "\u{2600}" } else { "\u{263E}" }}
                    </button>
                    <button class=BTN_GHOST on:click=move |_| on_add()>"Add"</button>
                    <button class=BTN_GHOST on:click=move |_| on_scan()>"Read Tag"</button>
                    <button class=BTN_GHOST on:click=move |_| on_settings()>"Settings"</button>
                </div>
            </div>
        </header>
    }.into_any()
}
