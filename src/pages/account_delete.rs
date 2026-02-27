use leptos::prelude::*;
use crate::components::botanical_art::OrchidSpray;
use crate::components::BTN_SECONDARY;
use crate::server_fns::auth::get_current_user;

const INPUT_CLASS: &str = "w-full px-4 py-3 text-sm bg-white/80 border border-stone-300/50 rounded-xl outline-none transition-all duration-200 placeholder:text-stone-500 focus:bg-white focus:border-primary/40 focus:ring-2 focus:ring-primary/10 dark:bg-stone-800/80 dark:border-stone-600/50 dark:placeholder:text-stone-400 dark:focus:bg-stone-800 dark:focus:border-primary-light/40 dark:focus:ring-primary-light/10";

#[component]
pub fn AccountDeletePage() -> impl IntoView {
    let user = Resource::new(|| (), |_| get_current_user());

    view! {
        <Suspense fallback=move || view! { <p class="p-8 text-center text-stone-500">"Loading..."</p> }>
            {move || {
                user.get().map(|result| match result {
                    Ok(Some(ref user_info)) => {
                        let username = user_info.username.clone();
                        view! { <AccountDeleteInner username=username /> }.into_any()
                    }
                    _ => {
                        #[cfg(feature = "ssr")]
                        leptos_axum::redirect("/login");
                        #[cfg(feature = "hydrate")]
                        {
                            if let Some(window) = web_sys::window() {
                                let _ = window.location().set_href("/login");
                            }
                        }
                        view! { <div></div> }.into_any()
                    }
                })
            }}
        </Suspense>
    }
}

#[component]
fn AccountDeleteInner(username: String) -> impl IntoView {
    let (step, set_step) = signal(1u8); // 1=warning, 2=confirm
    let (confirm_username, set_confirm_username) = signal(String::new());
    let (is_deleting, set_is_deleting) = signal(false);
    let (delete_error, set_delete_error) = signal(String::new());
    let username_stored = StoredValue::new(username);

    view! {
        <main class="flex min-h-screen bg-cream">
            // Left panel — botanical atmosphere (hidden on mobile)
            <div class="hidden overflow-hidden relative lg:flex lg:w-1/2 xl:w-3/5 bg-primary">
                <div class="absolute inset-0 bg-gradient-to-br from-primary via-primary-dark to-primary-dark"></div>
                <div class="absolute inset-0 auth-glow-green"></div>
                <div class="absolute inset-0 auth-glow-gold"></div>
                <div class="absolute inset-0 auth-grid opacity-[0.04]"></div>
                <div class="flex absolute inset-0 justify-center items-center botanical-draw text-white/[0.07]">
                    <OrchidSpray class="w-[75%] max-w-[360px] h-auto" />
                </div>
                <div class="flex relative z-10 flex-col justify-between p-12 xl:p-16">
                    // Top — brand
                    <div>
                        <div class="flex gap-3 items-center mb-2">
                            <div class="flex justify-center items-center w-10 h-10 text-lg rounded-xl border bg-white/10 border-white/20 [&>svg]:w-5 [&>svg]:h-5" inner_html=include_str!("../../public/svg/app_logo.svg")></div>
                            <div>
                                <span class="text-sm font-semibold tracking-widest uppercase text-white/70">"Velamen"</span>
                                <div class="text-xs italic tracking-wide text-white/40">"Root to Bloom"</div>
                            </div>
                        </div>
                    </div>

                    // Center — message about data ownership
                    <div class="max-w-lg">
                        <h1 class="mb-6 text-5xl leading-tight text-white xl:text-6xl">"Your data, your choice"</h1>
                        <p class="text-lg leading-relaxed text-white/80">
                            "We believe in data ownership. If you choose to leave, we\u{2019}ll remove everything\u{2014}no hidden copies, no retention. Your collection was always yours."
                        </p>
                    </div>

                    // Bottom — privacy detail
                    <div class="flex gap-8 items-center pt-8 border-t border-white/10">
                        <div>
                            <div class="text-2xl font-light text-accent-light">"GDPR"</div>
                            <div class="text-xs text-white/70">"Compliant"</div>
                        </div>
                        <div class="w-px h-8 bg-white/10"></div>
                        <div>
                            <div class="text-2xl font-light text-accent-light">"Zero"</div>
                            <div class="text-xs text-white/70">"Data Retained"</div>
                        </div>
                        <div class="w-px h-8 bg-white/10"></div>
                        <div>
                            <div class="text-2xl font-light text-accent-light">"Full"</div>
                            <div class="text-xs text-white/70">"Transparency"</div>
                        </div>
                    </div>
                </div>
            </div>

            // Right panel — deletion flow
            <div class="flex flex-col justify-center items-center px-6 w-full sm:px-12 lg:w-1/2 xl:w-2/5">
                <div class="w-full max-w-sm">
                    // Mobile brand (visible only on small screens)
                    <div class="flex gap-2 justify-center items-center mb-8 lg:hidden">
                        <div class="flex justify-center items-center w-8 h-8 text-sm rounded-lg bg-primary [&>svg]:w-4 [&>svg]:h-4" inner_html=include_str!("../../public/svg/app_logo.svg")></div>
                        <span class="text-sm font-semibold tracking-widest uppercase text-primary">"Velamen"</span>
                    </div>

                    {move || {
                        let current_step = step.get();
                        let uname = username_stored.get_value();

                        if current_step == 1 {
                            // Step 1: Warning
                            view! {
                                <div>
                                    <h2 class="mb-2 text-3xl text-red-700 dark:text-red-400">"Delete Your Account"</h2>
                                    <p class="mb-6 text-sm text-stone-500 dark:text-stone-400">"This will permanently delete all your data"</p>

                                    <div class="p-4 mb-6 rounded-xl border bg-red-50/50 border-red-200/60 dark:bg-red-950/20 dark:border-red-800/40">
                                        <p class="mb-3 text-sm font-medium text-stone-700 dark:text-stone-300">"The following will be permanently removed:"</p>
                                        <ul class="ml-4 space-y-1.5 text-sm list-disc text-stone-600 dark:text-stone-300">
                                            <li>"All your plants and care history"</li>
                                            <li>"Growing zones and climate readings"</li>
                                            <li>"Uploaded photos"</li>
                                            <li>"Hardware device credentials (Tempest, AC Infinity)"</li>
                                            <li>"Notification subscriptions"</li>
                                            <li>"All account settings and data"</li>
                                        </ul>
                                    </div>

                                    <p class="mb-8 text-sm font-semibold text-red-600 dark:text-red-400">"This action is permanent and cannot be undone."</p>

                                    <div class="flex gap-3">
                                        <a
                                            href="/"
                                            class=BTN_SECONDARY
                                        >"Cancel"</a>
                                        <button
                                            class="py-2.5 px-5 text-sm font-semibold text-white bg-red-600 rounded-lg border-none transition-colors cursor-pointer hover:bg-red-700"
                                            on:click=move |_| set_step.set(2)
                                        >"Continue"</button>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            // Step 2: Type username to confirm
                            let uname_clone = uname.clone();
                            let uname_display = uname.clone();
                            view! {
                                <div>
                                    <h2 class="mb-2 text-3xl text-red-700 dark:text-red-400">"This is irreversible"</h2>
                                    <p class="mb-6 text-sm text-stone-500 dark:text-stone-400">
                                        "To confirm, type your username "
                                        <strong class="text-stone-800 dark:text-stone-100">{uname_display}</strong>
                                        " below"
                                    </p>

                                    <div class="mb-6">
                                        <input
                                            type="text"
                                            class=INPUT_CLASS
                                            placeholder="Type your username"
                                            prop:value=move || confirm_username.get()
                                            on:input=move |ev| set_confirm_username.set(event_target_value(&ev))
                                            autocomplete="off"
                                        />
                                    </div>

                                    {move || {
                                        let err = delete_error.get();
                                        if err.is_empty() {
                                            None
                                        } else {
                                            Some(view! {
                                                <div class="flex gap-2 items-center p-3 mb-6 text-sm rounded-xl border animate-fade-in text-danger bg-danger/5 border-danger/10">
                                                    <svg xmlns="http://www.w3.org/2000/svg" class="flex-shrink-0 w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                                                        <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
                                                    </svg>
                                                    <span>{err}</span>
                                                </div>
                                            })
                                        }
                                    }}

                                    <button
                                        class="flex gap-2 justify-center items-center py-3 w-full text-sm font-semibold text-white bg-red-600 rounded-xl border-none transition-all duration-200 cursor-pointer hover:bg-red-700 hover:shadow-lg disabled:opacity-40 disabled:cursor-not-allowed hover:shadow-red-600/20 active:scale-[0.98] disabled:hover:bg-red-600 disabled:hover:shadow-none"
                                        disabled=move || confirm_username.get() != uname_clone || is_deleting.get()
                                        on:click=move |_| {
                                            let typed = confirm_username.get();
                                            set_is_deleting.set(true);
                                            set_delete_error.set(String::new());
                                            leptos::task::spawn_local(async move {
                                                match crate::server_fns::auth::delete_account(typed).await {
                                                    Ok(()) => {
                                                        #[cfg(feature = "hydrate")]
                                                        {
                                                            if let Some(window) = web_sys::window() {
                                                                let _ = window.location().set_href("/login");
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        set_delete_error.set(e.to_string());
                                                        set_is_deleting.set(false);
                                                    }
                                                }
                                            });
                                        }
                                    >
                                        {move || if is_deleting.get() {
                                            view! {
                                                <svg class="w-4 h-4 animate-spin" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
                                                </svg>
                                                <span>"Deleting..."</span>
                                            }.into_any()
                                        } else {
                                            view! { <span>"Delete My Account"</span> }.into_any()
                                        }}
                                    </button>

                                    <button
                                        class="py-2 mt-3 w-full text-sm font-medium bg-transparent border-none cursor-pointer text-stone-500 dark:text-stone-400 dark:hover:text-stone-200 hover:text-stone-700"
                                        on:click=move |_| {
                                            set_step.set(1);
                                            set_confirm_username.set(String::new());
                                            set_delete_error.set(String::new());
                                        }
                                    >"Go Back"</button>
                                </div>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </main>
    }
}
