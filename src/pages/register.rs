use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::components::botanical_art::OrchidSpray;
use crate::server_fns::auth::register;

const INPUT_CLASS: &str = "w-full px-4 py-3 text-sm bg-white/80 border border-stone-300/50 rounded-xl outline-none transition-all duration-200 placeholder:text-stone-400 focus:bg-white focus:border-primary/40 focus:ring-2 focus:ring-primary/10 dark:bg-stone-800/80 dark:border-stone-600/50 dark:placeholder:text-stone-500 dark:focus:bg-stone-800 dark:focus:border-primary-light/40 dark:focus:ring-primary-light/10";
const LABEL_CLASS: &str = "block mb-2 text-xs font-semibold tracking-widest uppercase text-stone-400 dark:text-stone-500";

#[component]
pub fn RegisterPage() -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (confirm, set_confirm) = signal(String::new());
    let (error, set_error) = signal::<Option<String>>(None);
    let (is_loading, set_is_loading) = signal(false);
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        if password.get() != confirm.get() {
            set_error.set(Some("Passwords do not match".into()));
            return;
        }

        set_is_loading.set(true);
        set_error.set(None);

        let nav = navigate.clone();
        leptos::task::spawn_local(async move {
            match register(username.get(), email.get(), password.get()).await {
                Ok(_) => {
                    nav("/", Default::default());
                }
                Err(e) => {
                    set_error.set(Some(e.to_string()));
                    set_is_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="flex min-h-screen bg-cream">
            // Left panel — botanical atmosphere (hidden on mobile)
            <div class="hidden overflow-hidden relative lg:flex lg:w-1/2 xl:w-3/5 bg-primary">
                <div class="absolute inset-0 bg-gradient-to-br from-primary-dark via-primary to-primary-light/80"></div>
                <div class="absolute inset-0 auth-glow-green-alt"></div>
                <div class="absolute inset-0 auth-glow-gold-alt"></div>
                <div class="absolute inset-0 auth-grid opacity-[0.04]"></div>
                <div class="flex absolute inset-0 justify-center items-center botanical-draw text-white/[0.07]" style="transform: scaleX(-1)">
                    <OrchidSpray class="w-[75%] max-w-[360px] h-auto" />
                </div>
                <div class="flex relative z-10 flex-col justify-between p-12 xl:p-16">
                    // Top — brand
                    <div>
                        <div class="flex gap-3 items-center mb-2">
                            <div class="flex justify-center items-center w-10 h-10 text-lg rounded-xl border bg-white/10 border-white/20 [&>svg]:w-5 [&>svg]:h-5" inner_html=include_str!("../../public/svg/app_logo.svg")></div>
                            <span class="text-sm font-semibold tracking-widest uppercase text-white/70">"Orchid Tracker"</span>
                        </div>
                    </div>

                    // Center — hero text
                    <div class="max-w-lg">
                        <h1 class="mb-6 text-5xl leading-tight text-white xl:text-6xl">"Start your collection"</h1>
                        <p class="text-lg leading-relaxed text-white/60">
                            "Join a growing community of orchid enthusiasts. Catalog your plants, track their care, and discover new species with AI assistance."
                        </p>
                    </div>

                    // Bottom — feature highlights
                    <div class="flex gap-8 items-center pt-8 border-t border-white/10">
                        <div>
                            <div class="flex gap-2 items-center mb-1">
                                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 text-accent-light" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg>
                                <span class="text-sm font-medium text-white/80">"Free forever"</span>
                            </div>
                            <div class="text-xs text-white/40">"No credit card needed"</div>
                        </div>
                        <div class="w-px h-8 bg-white/10"></div>
                        <div>
                            <div class="flex gap-2 items-center mb-1">
                                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 text-accent-light" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                                </svg>
                                <span class="text-sm font-medium text-white/80">"Self-hosted"</span>
                            </div>
                            <div class="text-xs text-white/40">"Your data stays yours"</div>
                        </div>
                    </div>
                </div>
            </div>

            // Right panel — register form
            <div class="flex flex-col justify-center items-center px-6 w-full sm:px-12 lg:w-1/2 xl:w-2/5">
                <div class="w-full max-w-sm">
                    // Mobile brand (visible only on small screens)
                    <div class="flex gap-2 justify-center items-center mb-8 lg:hidden">
                        <div class="flex justify-center items-center w-8 h-8 text-sm rounded-lg bg-primary [&>svg]:w-4 [&>svg]:h-4" inner_html=include_str!("../../public/svg/app_logo.svg")></div>
                        <span class="text-sm font-semibold tracking-widest uppercase text-primary">"Orchid Tracker"</span>
                    </div>

                    <div class="mb-2">
                        <h2 class="text-3xl text-stone-800 dark:text-stone-100">"Create account"</h2>
                    </div>
                    <p class="mb-8 text-sm text-stone-400 dark:text-stone-500">"Set up your orchid collection"</p>

                    {move || error.get().map(|err| view! {
                        <div class="flex gap-2 items-center p-3 mb-6 text-sm rounded-xl border animate-fade-in text-danger bg-danger/5 border-danger/10">
                            <svg xmlns="http://www.w3.org/2000/svg" class="flex-shrink-0 w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
                            </svg>
                            <span>{err}</span>
                        </div>
                    })}

                    <form on:submit=on_submit>
                        <div class="mb-5">
                            <label class=LABEL_CLASS>"Username"</label>
                            <input
                                type="text"
                                class=INPUT_CLASS
                                placeholder="Choose a username"
                                prop:value=username
                                on:input=move |ev| set_username.set(event_target_value(&ev))
                                required
                                autocomplete="username"
                            />
                            <p class="mt-1.5 text-xs text-stone-400/80">"Letters, numbers, hyphens, and underscores"</p>
                        </div>
                        <div class="mb-5">
                            <label class=LABEL_CLASS>"Email"</label>
                            <input
                                type="email"
                                class=INPUT_CLASS
                                placeholder="your@email.com"
                                prop:value=email
                                on:input=move |ev| set_email.set(event_target_value(&ev))
                                required
                                autocomplete="email"
                            />
                        </div>
                        <div class="mb-5">
                            <label class=LABEL_CLASS>"Password"</label>
                            <input
                                type="password"
                                class=INPUT_CLASS
                                placeholder="Minimum 8 characters"
                                prop:value=password
                                on:input=move |ev| set_password.set(event_target_value(&ev))
                                required
                                minlength="8"
                                autocomplete="new-password"
                            />
                        </div>
                        <div class="mb-8">
                            <label class=LABEL_CLASS>"Confirm Password"</label>
                            <input
                                type="password"
                                class=INPUT_CLASS
                                placeholder="Re-enter your password"
                                prop:value=confirm
                                on:input=move |ev| set_confirm.set(event_target_value(&ev))
                                required
                                autocomplete="new-password"
                            />
                        </div>
                        <button
                            type="submit"
                            class="flex gap-2 justify-center items-center py-3 w-full text-sm font-semibold text-white rounded-xl border-none transition-all duration-200 cursor-pointer hover:shadow-lg disabled:opacity-50 disabled:cursor-not-allowed bg-primary hover:bg-primary-dark hover:shadow-primary/20 active:scale-[0.98]"
                            disabled=move || is_loading.get()
                        >
                            {move || if is_loading.get() {
                                view! {
                                    <svg class="w-4 h-4 animate-spin" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
                                    </svg>
                                    <span>"Creating account..."</span>
                                }.into_any()
                            } else {
                                view! { <span>"Create Account"</span> }.into_any()
                            }}
                        </button>
                    </form>

                    <div class="flex gap-1 justify-center items-center mt-8 text-sm">
                        <span class="text-stone-400 dark:text-stone-500">"Already have an account?"</span>
                        <a href="/login" class="font-medium transition-colors text-primary dark:text-primary-light dark:hover:text-accent-light hover:text-primary-light">"Sign in"</a>
                    </div>
                </div>
            </div>
        </div>
    }
}
