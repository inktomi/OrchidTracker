use leptos::prelude::*;

/// Client-side component that handles push notification permission + subscription.
/// Only renders meaningful UI on the hydrate target.
#[component]
pub fn NotificationSetup() -> impl IntoView {
    let (show_banner, set_show_banner) = signal(false);
    let (is_subscribing, set_is_subscribing) = signal(false);

    #[cfg(feature = "hydrate")]
    {
        Effect::new(move |_| {
            let perm = web_sys::Notification::permission();
            if perm == web_sys::NotificationPermission::Default {
                set_show_banner.set(true);
            } else if perm == web_sys::NotificationPermission::Granted {
                leptos::task::spawn_local(async move {
                    register_and_subscribe_silent().await;
                });
            }
        });
    }

    let on_allow = move |_| {
        set_is_subscribing.set(true);
        #[cfg(feature = "hydrate")]
        {
            leptos::task::spawn_local(async move {
                use wasm_bindgen_futures::JsFuture;

                if let Ok(promise) = web_sys::Notification::request_permission() {
                    let _ = JsFuture::from(promise).await;
                }

                if web_sys::Notification::permission() == web_sys::NotificationPermission::Granted {
                    register_and_subscribe_silent().await;
                }

                set_show_banner.set(false);
                set_is_subscribing.set(false);
            });
        }
        #[cfg(not(feature = "hydrate"))]
        {
            set_is_subscribing.set(false);
        }
    };

    let on_dismiss = move |_| {
        set_show_banner.set(false);
    };

    view! {
        {move || show_banner.get().then(|| {
            view! {
                <div class="flex gap-3 justify-between items-center p-3 mb-4 text-sm rounded-xl border bg-sky-50 border-sky-200 dark:bg-sky-900/20 dark:border-sky-800">
                    <span class="text-sky-700 dark:text-sky-300">"Enable notifications for care alerts?"</span>
                    <div class="flex gap-2">
                        <button
                            class="py-1.5 px-3 text-xs font-semibold text-white rounded-lg border-none transition-colors cursor-pointer bg-primary hover:bg-primary-dark"
                            disabled=move || is_subscribing.get()
                            on:click=on_allow
                        >"Allow"</button>
                        <button
                            class="py-1.5 px-3 text-xs font-medium rounded-lg border-none transition-colors cursor-pointer text-stone-500 bg-stone-100 dark:text-stone-400 dark:bg-stone-700 dark:hover:bg-stone-600 hover:bg-stone-200"
                            on:click=on_dismiss
                        >"Dismiss"</button>
                    </div>
                </div>
            }
        })}
    }
}

/// Public entry point for registering push subscription from settings.
/// Returns Ok(()) on success or Err with a user-visible error message.
#[cfg(feature = "hydrate")]
pub async fn register_and_subscribe_from_settings() -> Result<(), String> {
    register_and_subscribe().await
}

/// Register the service worker and subscribe to push notifications.
#[cfg(feature = "hydrate")]
async fn register_and_subscribe() -> Result<(), String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or("No window object")?;
    let navigator = window.navigator();
    let sw_container = navigator.service_worker();

    // Register service worker
    let promise = sw_container.register("/sw.js");
    let registration = JsFuture::from(promise).await
        .map_err(|e| format!("Service worker registration failed: {:?}", e))?
        .dyn_into::<web_sys::ServiceWorkerRegistration>()
        .map_err(|_| "Service worker registration returned unexpected type".to_string())?;

    // Get VAPID public key
    let vapid_key = crate::server_fns::alerts::get_vapid_public_key().await
        .map_err(|e| format!("Failed to get VAPID key: {}", e))?;

    if vapid_key.is_empty() {
        return Err("VAPID key not configured on server".into());
    }

    // Convert VAPID key from URL-safe base64 to Uint8Array
    let key_bytes = url_safe_base64_decode(&vapid_key);
    let key_array = js_sys::Uint8Array::from(key_bytes.as_slice());

    // Subscribe to push
    let push_manager = registration.push_manager()
        .map_err(|e| format!("Push manager error: {:?}", e))?;

    let opts = web_sys::PushSubscriptionOptionsInit::new();
    opts.set_user_visible_only(true);
    opts.set_application_server_key(&key_array);

    let promise = push_manager.subscribe_with_options(&opts)
        .map_err(|e| format!("Push subscribe error: {:?}", e))?;

    let subscription = JsFuture::from(promise).await
        .map_err(|e| format!("Push subscribe failed: {:?}", e))?
        .dyn_into::<web_sys::PushSubscription>()
        .map_err(|_| "Push subscription returned unexpected type".to_string())?;

    // Extract subscription details via to_json()
    let sub_json = subscription.to_json()
        .map_err(|e| format!("Subscription to_json error: {:?}", e))?;

    // Parse the JS object to extract endpoint, keys
    let endpoint = js_sys::Reflect::get(&sub_json, &"endpoint".into())
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default();

    let keys = js_sys::Reflect::get(&sub_json, &"keys".into()).ok();
    let p256dh = keys.as_ref()
        .and_then(|k| js_sys::Reflect::get(k, &"p256dh".into()).ok())
        .and_then(|v| v.as_string())
        .unwrap_or_default();
    let auth = keys.as_ref()
        .and_then(|k| js_sys::Reflect::get(k, &"auth".into()).ok())
        .and_then(|v| v.as_string())
        .unwrap_or_default();

    if endpoint.is_empty() || p256dh.is_empty() || auth.is_empty() {
        return Err("Push subscription missing endpoint/keys".into());
    }

    // Send subscription to server
    crate::server_fns::alerts::subscribe_push(endpoint, p256dh, auth).await
        .map_err(|e| format!("Failed to store subscription: {}", e))?;

    Ok(())
}

/// Silent version for the initial notification setup banner (doesn't need error reporting).
#[cfg(feature = "hydrate")]
pub(crate) async fn register_and_subscribe_silent() {
    if let Err(e) = register_and_subscribe().await {
        log::error!("Push subscribe failed: {}", e);
    }
}

#[cfg(feature = "hydrate")]
fn url_safe_base64_decode(input: &str) -> Vec<u8> {
    // URL-safe base64 decode (no padding) using browser's atob
    let padded = match input.len() % 4 {
        2 => format!("{}==", input),
        3 => format!("{}=", input),
        _ => input.to_string(),
    };
    let standard = padded.replace('-', "+").replace('_', "/");
    if let Some(window) = web_sys::window() {
        match window.atob(&standard) {
            Ok(decoded) => decoded.bytes().collect(),
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    }
}
