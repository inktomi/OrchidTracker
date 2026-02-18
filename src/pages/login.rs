use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::server_fns::auth::login;

const INPUT_CLASS: &str = "w-full px-4 py-3 text-sm bg-white/80 border border-stone-300/50 rounded-xl outline-none transition-all duration-200 placeholder:text-stone-400 focus:bg-white focus:border-primary/40 focus:ring-2 focus:ring-primary/10 dark:bg-stone-800/80 dark:border-stone-600/50 dark:placeholder:text-stone-500 dark:focus:bg-stone-800 dark:focus:border-primary-light/40 dark:focus:ring-primary-light/10";
const LABEL_CLASS: &str = "block mb-2 text-xs font-semibold tracking-widest uppercase text-stone-400 dark:text-stone-500";

#[component]
pub fn LoginPage() -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal::<Option<String>>(None);
    let (is_loading, set_is_loading) = signal(false);
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_is_loading.set(true);
        set_error.set(None);

        let nav = navigate.clone();
        leptos::task::spawn_local(async move {
            match login(username.get(), password.get()).await {
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
                // Layered gradient background
                <div class="absolute inset-0 bg-gradient-to-br from-primary via-primary-dark to-primary-dark"></div>
                <div class="absolute inset-0 bg-[radial-gradient(ellipse_at_20%_50%,rgba(45,106,79,0.6),transparent_70%)]"></div>
                <div class="absolute inset-0 bg-[radial-gradient(ellipse_at_80%_20%,rgba(182,141,64,0.15),transparent_50%)]"></div>

                // Decorative grid pattern
                <div class="absolute inset-0 opacity-[0.04]"
                     style="background-image: linear-gradient(rgba(255,255,255,0.3) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.3) 1px, transparent 1px); background-size: 60px 60px;">
                </div>

                // Content
                <div class="flex relative z-10 flex-col justify-between p-12 xl:p-16">
                    // Top — brand
                    <div>
                        <div class="flex gap-3 items-center mb-2">
                            <div class="flex justify-center items-center w-10 h-10 text-lg rounded-xl border bg-white/10 border-white/20">"🌿"</div>
                            <span class="text-sm font-semibold tracking-widest uppercase text-white/70">"Orchid Tracker"</span>
                        </div>
                    </div>

                    // Center — hero text
                    <div class="max-w-lg">
                        <h1 class="mb-6 text-5xl leading-tight text-white xl:text-6xl">"Cultivate with confidence"</h1>
                        <p class="text-lg leading-relaxed text-white/60">
                            "Track your orchid collection, monitor growing conditions, and get AI-powered care recommendations\u{2014}all in one place."
                        </p>
                    </div>

                    // Bottom — decorative detail
                    <div class="flex gap-8 items-center pt-8 border-t border-white/10">
                        <div>
                            <div class="text-2xl font-light text-accent-light">"AI"</div>
                            <div class="text-xs text-white/40">"Species ID"</div>
                        </div>
                        <div class="w-px h-8 bg-white/10"></div>
                        <div>
                            <div class="text-2xl font-light text-accent-light">"24/7"</div>
                            <div class="text-xs text-white/40">"Climate Data"</div>
                        </div>
                        <div class="w-px h-8 bg-white/10"></div>
                        <div>
                            <div class="text-2xl font-light text-accent-light">"CITES"</div>
                            <div class="text-xs text-white/40">"Tracking"</div>
                        </div>
                    </div>
                </div>
            </div>

            // Right panel — login form
            <div class="flex flex-col justify-center items-center px-6 w-full sm:px-12 lg:w-1/2 xl:w-2/5">
                <div class="w-full max-w-sm">
                    // Mobile brand (visible only on small screens)
                    <div class="flex gap-2 justify-center items-center mb-8 lg:hidden">
                        <div class="flex justify-center items-center w-8 h-8 text-sm rounded-lg bg-primary">"🌿"</div>
                        <span class="text-sm font-semibold tracking-widest uppercase text-primary">"Orchid Tracker"</span>
                    </div>

                    <div class="mb-2">
                        <h2 class="text-3xl text-stone-800 dark:text-stone-100">"Welcome back"</h2>
                    </div>
                    <p class="mb-8 text-sm text-stone-400 dark:text-stone-500">"Sign in to your collection"</p>

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
                                placeholder="Enter your username"
                                prop:value=username
                                on:input=move |ev| set_username.set(event_target_value(&ev))
                                required
                                autocomplete="username"
                            />
                        </div>
                        <div class="mb-8">
                            <label class=LABEL_CLASS>"Password"</label>
                            <input
                                type="password"
                                class=INPUT_CLASS
                                placeholder="Enter your password"
                                prop:value=password
                                on:input=move |ev| set_password.set(event_target_value(&ev))
                                required
                                autocomplete="current-password"
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
                                    <span>"Signing in..."</span>
                                }.into_any()
                            } else {
                                view! { <span>"Sign In"</span> }.into_any()
                            }}
                        </button>
                    </form>

                    <div class="flex gap-1 justify-center items-center mt-8 text-sm">
                        <span class="text-stone-400 dark:text-stone-500">"New to Orchid Tracker?"</span>
                        <a href="/register" class="font-medium transition-colors text-primary dark:text-primary-light dark:hover:text-accent-light hover:text-primary-light">"Create account"</a>
                    </div>
                </div>
            </div>
        </div>
    }
}
