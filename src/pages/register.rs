use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::server_fns::auth::register;
use crate::components::BTN_PRIMARY;

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
        <div class="flex justify-center items-center px-4 min-h-screen bg-cream">
            <div class="p-8 w-full max-w-md rounded-2xl border shadow-lg bg-surface border-stone-200/60 dark:border-stone-700/60">
                <h1 class="mb-2 text-2xl text-center text-primary">"Orchid Tracker"</h1>
                <p class="mb-6 text-sm text-center text-stone-500">"Create an account"</p>

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
                    <div class="mb-4">
                        <label>"Email"</label>
                        <input
                            type="email"
                            prop:value=email
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                            required
                            autocomplete="email"
                        />
                    </div>
                    <div class="mb-4">
                        <label>"Password (min 8 characters)"</label>
                        <input
                            type="password"
                            prop:value=password
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                            required
                            minlength="8"
                            autocomplete="new-password"
                        />
                    </div>
                    <div class="mb-6">
                        <label>"Confirm Password"</label>
                        <input
                            type="password"
                            prop:value=confirm
                            on:input=move |ev| set_confirm.set(event_target_value(&ev))
                            required
                            autocomplete="new-password"
                        />
                    </div>
                    <button type="submit" class=format!("{} w-full", BTN_PRIMARY) disabled=move || is_loading.get()>
                        {move || if is_loading.get() { "Creating account..." } else { "Register" }}
                    </button>
                </form>

                <p class="mt-4 text-sm text-center text-stone-500">
                    "Already have an account? "
                    <a href="/login" class="font-medium hover:underline text-primary">"Sign in"</a>
                </p>
            </div>
        </div>
    }
}
