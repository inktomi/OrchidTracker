use leptos::prelude::*;

#[component]
pub fn FirstBloomCelebration(
    on_dismiss: impl Fn() + 'static + Clone + Send + Sync,
) -> impl IntoView {
    let on_dismiss2 = on_dismiss.clone();

    // Auto-dismiss after 4 seconds
    #[cfg(feature = "hydrate")]
    {
        let dismiss = on_dismiss.clone();
        leptos::task::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(4000).await;
            dismiss();
        });
    }

    view! {
        <div
            class="flex fixed inset-0 flex-col gap-4 justify-center items-center cursor-pointer z-[3000] bloom-overlay"
            on:click=move |_| on_dismiss2()
        >
            // Golden glow background
            <div class="absolute inset-0 bloom-glow"></div>

            // Petal confetti
            <div class="overflow-hidden absolute inset-0 pointer-events-none">
                {(0..8).map(|i| {
                    let delay = format!("animation-delay: {}ms", i * 200);
                    let left = format!("left: {}%", 10 + (i * 11) % 80);
                    view! {
                        <span
                            class="absolute top-0 text-2xl petal-fall"
                            style=format!("{}; {}", delay, left)
                        >"\u{1F33A}"</span>
                    }
                }).collect::<Vec<_>>()}
            </div>

            // Main text
            <div class="relative z-10 text-center bloom-badge-in">
                <div class="mb-4 text-5xl">"\u{1F33C}"</div>
                <h2 class="m-0 text-3xl text-amber-800 dark:text-amber-200 font-display">"First Bloom!"</h2>
                <p class="mt-2 text-sm text-amber-700/80 dark:text-amber-300/80">"A milestone in your orchid's journey"</p>
            </div>
        </div>
    }.into_any()
}
