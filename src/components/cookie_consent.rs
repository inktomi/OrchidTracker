use leptos::prelude::*;

/// A cookie consent banner shown at the bottom of the screen on first visit.
/// Since Velamen only uses essential cookies, this is informational — it lets
/// users acknowledge the notice and links to the full cookie policy.
/// Consent state is stored in localStorage (not a cookie) under `velamen_cookie_consent`.
#[component]
pub fn CookieConsent() -> impl IntoView {
    let (visible, set_visible) = signal(false);

    // On mount, check localStorage to see if user already acknowledged
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let already_accepted = storage
                        .get_item("velamen_cookie_consent")
                        .ok()
                        .flatten()
                        .is_some();
                    if !already_accepted {
                        set_visible.set(true);
                    }
                }
            }
        });
    }

    let accept = move |_| {
        #[cfg(feature = "hydrate")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item("velamen_cookie_consent", "accepted");
                }
            }
        }
        set_visible.set(false);
    };

    view! {
        {move || {
            if !visible.get() {
                return None;
            }
            Some(view! {
                <div class="fixed right-0 bottom-0 left-0 z-[1100] animate-fade-in">
                    <div class="py-3 px-4 mx-4 mx-auto mb-4 max-w-xl rounded-xl border shadow-lg sm:py-4 sm:px-6 sm:mx-auto sm:mb-6 bg-surface border-stone-200/80 dark:border-stone-700/80">
                        <div class="flex flex-col gap-3 sm:flex-row sm:gap-4 sm:items-center">
                            <div class="flex-1 text-sm text-stone-600 dark:text-stone-300">
                                "We use a single essential cookie to keep you signed in. No tracking or advertising cookies are used. "
                                <a href="/cookie-policy" class="font-medium underline transition-colors text-primary dark:text-primary-light dark:hover:text-accent-light hover:text-primary-light">"Learn more"</a>
                            </div>
                            <button
                                class="flex-shrink-0 py-2 px-5 text-sm font-semibold text-white rounded-lg border-none transition-colors cursor-pointer bg-primary hover:bg-primary-dark"
                                on:click=accept
                            >"Got it"</button>
                        </div>
                    </div>
                </div>
            })
        }}
    }
}
