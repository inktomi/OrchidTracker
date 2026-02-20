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
    JsFuture::from(promise).await
        .map_err(|e| format!("Service worker registration failed: {:?}", e))?;

    // Wait for the SW to be active — register() resolves before activation,
    // but push subscriptions must come from an active registration
    let ready_promise = sw_container.ready()
        .map_err(|e| format!("SW ready error: {:?}", e))?;
    let registration = JsFuture::from(ready_promise).await
        .map_err(|e| format!("SW not ready: {:?}", e))?
        .dyn_into::<web_sys::ServiceWorkerRegistration>()
        .map_err(|_| "SW ready returned unexpected type".to_string())?;

    // Get VAPID public key
    let vapid_key = crate::server_fns::alerts::get_vapid_public_key().await
        .map_err(|e| format!("Failed to get VAPID key: {}", e))?;

    if vapid_key.is_empty() {
        return Err("VAPID key not configured on server".into());
    }

    // Convert VAPID key from URL-safe base64 to Uint8Array
    let key_bytes = base64url_decode(&vapid_key);
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

/// Decode a base64url (RFC 4648 §5) string to raw bytes.
/// Accepts both URL-safe (`-_`) and standard (`+/`) alphabets, with or without padding.
/// Pure Rust — no browser APIs required, so it's testable everywhere.
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) fn base64url_decode(input: &str) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;

    for &byte in input.as_bytes() {
        let val = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'-' | b'+' => 62,
            b'_' | b'/' => 63,
            _ => continue, // skip padding '=' and whitespace
        };
        buf = (buf << 6) | val as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::base64url_decode;

    #[test]
    fn test_base64url_decode_empty() {
        assert_eq!(base64url_decode(""), Vec::<u8>::new());
    }

    #[test]
    fn test_base64url_decode_hello() {
        // "Hello" = SGVsbG8 in base64url (no padding)
        assert_eq!(base64url_decode("SGVsbG8"), b"Hello".to_vec());
    }

    #[test]
    fn test_base64url_decode_with_padding() {
        // "Hello" = SGVsbG8= with standard base64 padding
        assert_eq!(base64url_decode("SGVsbG8="), b"Hello".to_vec());
    }

    #[test]
    fn test_base64url_decode_high_bytes() {
        // Bytes > 127 must decode correctly (this was the atob/UTF-8 bug).
        // 0xFF 0xFE = //4 in standard base64, __4 in base64url
        assert_eq!(base64url_decode("__4"), vec![0xFF, 0xFE]);
        // Same with standard alphabet
        assert_eq!(base64url_decode("//4"), vec![0xFF, 0xFE]);
    }

    #[test]
    fn test_base64url_decode_65_byte_p256_key() {
        // A VAPID key is 65 bytes: 0x04 + 32 X + 32 Y.
        // Generate a known 65-byte payload and verify round-trip length.
        let key_bytes: Vec<u8> = (0..65).map(|i| (i * 3 + 7) as u8).collect();
        // Encode with standard base64 then make URL-safe
        let encoded = standard_base64_encode(&key_bytes)
            .replace('+', "-")
            .replace('/', "_")
            .trim_end_matches('=')
            .to_string();
        let decoded = base64url_decode(&encoded);
        assert_eq!(decoded.len(), 65, "P-256 key must decode to exactly 65 bytes");
        assert_eq!(decoded, key_bytes);
    }

    /// Minimal base64 encoder for test use only.
    fn standard_base64_encode(data: &[u8]) -> String {
        const TABLE: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for chunk in data.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
            let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
            let triple = (b0 << 16) | (b1 << 8) | b2;
            out.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
            out.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);
            if chunk.len() > 1 {
                out.push(TABLE[((triple >> 6) & 0x3F) as usize] as char);
            } else {
                out.push('=');
            }
            if chunk.len() > 2 {
                out.push(TABLE[(triple & 0x3F) as usize] as char);
            } else {
                out.push('=');
            }
        }
        out
    }
}
