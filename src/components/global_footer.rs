use leptos::prelude::*;

const FOOTER_LINK: &str = "transition-colors hover:text-stone-600 dark:hover:text-stone-300";

#[component]
pub fn GlobalFooter() -> impl IntoView {
    view! {
        <footer class="py-6 mt-auto bg-cream">
            <div class="px-6 mx-auto max-w-2xl sm:px-8">
                <div class="pt-4 border-t border-stone-200 dark:border-stone-700">
                    <div class="flex flex-col gap-3 justify-between items-center sm:flex-row">
                        <p class="text-xs text-stone-400 dark:text-stone-500">
                            {format!("\u{00a9} 2026 Velamen. All rights reserved. v{}", env!("CARGO_PKG_VERSION"))}
                        </p>
                        <nav class="flex gap-4 text-xs text-stone-400 dark:text-stone-500">
                            <a href="/terms" class=FOOTER_LINK>"Terms of Service"</a>
                            <span class="text-stone-300 dark:text-stone-600">"\u{00b7}"</span>
                            <a href="/cookie-policy" class=FOOTER_LINK>"Cookie Policy"</a>
                        </nav>
                    </div>
                </div>
            </div>
        </footer>
    }
}
