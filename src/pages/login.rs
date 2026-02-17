use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::server_fns::auth::login;
use crate::components::BTN_PRIMARY;

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
        <div class="flex justify-center items-center px-4 min-h-screen bg-cream">
            <div class="p-8 w-full max-w-md rounded-2xl border shadow-lg bg-surface border-stone-200/60 dark:border-stone-700/60">
                <h1 class="mb-2 text-2xl text-center text-primary">"Orchid Tracker"</h1>
                <p class="mb-6 text-sm text-center text-stone-500">"Sign in to manage your collection"</p>

                {move || error.get().map(|err| view! {
                    <div class="p-3 mb-4 text-sm rounded-lg text-danger bg-danger/10">{err}</div>
                })}

                <form on:submit=on_submit>
                    <div class="mb-4">
                        <label>"Username"</label>
                        <input
                            type="text"
                            prop:value=username
                            on:input=move |ev| set_username.set(event_target_value(&ev))
                            required
                            autocomplete="username"
                        />
                    </div>
                    <div class="mb-6">
                        <label>"Password"</label>
                        <input
                            type="password"
                            prop:value=password
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                            required
                            autocomplete="current-password"
                        />
                    </div>
                    <button type="submit" class=format!("{} w-full", BTN_PRIMARY) disabled=move || is_loading.get()>
                        {move || if is_loading.get() { "Signing in..." } else { "Sign In" }}
                    </button>
                </form>

                <p class="mt-4 text-sm text-center text-stone-500">
                    "Don't have an account? "
                    <a href="/register" class="font-medium hover:underline text-primary">"Register"</a>
                </p>
            </div>
        </div>
    }
}
