use leptos::prelude::*;

/// Max dimension (width or height) for resized images.
/// 2048px preserves good detail while keeping JPEG size well under 2MB.
#[cfg(feature = "hydrate")]
const MAX_IMAGE_DIMENSION: u32 = 2048;

/// Upload a JPEG data URL to the server. Returns the server filename on success.
/// Called by the parent form on submit (not by PhotoCapture itself).
#[cfg(feature = "hydrate")]
pub async fn upload_data_url(data_url: &str) -> Result<String, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or("no window")?;

    // Convert data URL to Blob
    let resp_val = JsFuture::from(window.fetch_with_str(data_url))
        .await
        .map_err(|_| "fetch data URL failed")?;
    let resp: web_sys::Response = resp_val
        .dyn_into()
        .map_err(|_| "cast response failed")?;
    let blob_val = JsFuture::from(
        resp.blob().map_err(|_| "blob() failed")?
    )
        .await
        .map_err(|_| "blob await failed")?;
    let image_blob: web_sys::Blob = blob_val
        .dyn_into()
        .map_err(|_| "cast blob failed")?;

    // Build multipart form and upload
    let form_data = web_sys::FormData::new()
        .map_err(|_| "Failed to create form data")?;
    let _ = form_data.append_with_blob_and_filename("image", &image_blob, "photo.jpg");

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&form_data.into());

    let request = web_sys::Request::new_with_str_and_init("/api/images/upload", &opts)
        .map_err(|_| "Failed to create request")?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| "Upload failed")?;

    let upload_resp: web_sys::Response = resp_value
        .dyn_into()
        .map_err(|_| "Invalid response")?;

    if !upload_resp.ok() {
        return Err(format!("Upload error: {}", upload_resp.status()));
    }

    let json = JsFuture::from(
        upload_resp.json().map_err(|_| "Failed to read response")?
    )
        .await
        .map_err(|_| "Failed to parse response")?;

    js_sys::Reflect::get(&json, &"filename".into())
        .ok()
        .and_then(|v| v.as_string())
        .ok_or_else(|| "No filename in response".to_string())
}

#[component]
pub fn PhotoCapture(
    /// Called with a JPEG data URL when a photo is staged locally (not yet uploaded).
    on_photo_ready: impl Fn(String) + 'static + Copy + Send + Sync,
    #[prop(optional)] on_clear: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
    /// Bump this signal to reset the component (clear preview after successful save).
    #[prop(optional)] reset: Option<ReadSignal<u32>>,
) -> impl IntoView {
    let (preview_data_url, set_preview_data_url) = signal(Option::<String>::None);
    let (is_processing, set_is_processing) = signal(false);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (is_dragging, set_is_dragging) = signal(false);
    let file_input_ref = NodeRef::<leptos::html::Input>::new();
    let on_clear_stored = StoredValue::new(on_clear);
    // These are only used in #[cfg(feature = "hydrate")] blocks
    let _ = &on_photo_ready;
    let _ = &set_is_processing;
    let _ = &set_error_msg;

    // Watch reset signal from parent to clear preview after save
    if let Some(reset_signal) = reset {
        Effect::new(move |prev: Option<u32>| {
            let current = reset_signal.get();
            if let Some(prev_val) = prev
                && current != prev_val
            {
                set_preview_data_url.set(None);
                set_error_msg.set(None);
            }
            current
        });
    }

    // Resize the image client-side using canvas to produce a data URL for local preview.
    // Upload is deferred until the parent form is submitted.
    #[cfg(feature = "hydrate")]
    let stage_photo = move |file: web_sys::File| {
        set_is_processing.set(true);
        set_error_msg.set(None);

        leptos::task::spawn_local(async move {
            // Load the file into an image element for resizing
            let blob_url = match web_sys::Url::create_object_url_with_blob(&file) {
                Ok(u) => u,
                Err(_) => {
                    set_error_msg.set(Some("Failed to read image file".into()));
                    set_is_processing.set(false);
                    return;
                }
            };

            match resize_to_data_url(&blob_url).await {
                Ok(data_url) => {
                    let _ = web_sys::Url::revoke_object_url(&blob_url);
                    set_preview_data_url.set(Some(data_url.clone()));
                    on_photo_ready(data_url);
                }
                Err(e) => {
                    let _ = web_sys::Url::revoke_object_url(&blob_url);
                    tracing::warn!("Image resize failed: {}", e);
                    set_error_msg.set(Some("Failed to process image".into()));
                }
            }
            set_is_processing.set(false);
        });
    };

    let on_file_change = move |_ev: leptos::ev::Event| {
        #[cfg(feature = "hydrate")]
        {
            if let Some(input) = file_input_ref.get() {
                let input_el: &web_sys::HtmlInputElement = input.as_ref();
                if let Some(files) = input_el.files() {
                    if let Some(file) = files.get(0) {
                        stage_photo(file);
                    }
                }
            }
        }
    };

    let on_drop = move |ev: leptos::ev::DragEvent| {
        ev.prevent_default();
        set_is_dragging.set(false);
        #[cfg(feature = "hydrate")]
        {
            if let Some(dt) = ev.data_transfer() {
                if let Some(files) = dt.files() {
                    if let Some(file) = files.get(0) {
                        stage_photo(file);
                    }
                }
            }
        }
    };

    let on_dragover = move |ev: leptos::ev::DragEvent| {
        ev.prevent_default();
        set_is_dragging.set(true);
    };

    let on_dragleave = move |ev: leptos::ev::DragEvent| {
        ev.prevent_default();
        set_is_dragging.set(false);
    };

    let clear_photo = move |_| {
        set_preview_data_url.set(None);
        on_clear_stored.with_value(|oc| {
            if let Some(cb) = oc {
                cb();
            }
        });
    };

    view! {
        <div>
            {move || {
                if let Some(data_url) = preview_data_url.get() {
                    view! {
                        <div class="inline-block relative">
                            <img
                                src=data_url
                                class="block max-w-full rounded-lg border max-h-[200px] border-stone-200 dark:border-stone-700"
                                alt="Photo preview"
                            />
                            <button
                                type="button"
                                class="flex absolute top-1 right-1 justify-center items-center w-6 h-6 text-xs font-bold text-white rounded-full border-none cursor-pointer bg-danger hover:bg-danger-dark"
                                on:click=clear_photo
                            >
                                "\u{00D7}"
                            </button>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div
                            class=move || {
                                if is_dragging.get() {
                                    "flex flex-col gap-2 justify-center items-center p-6 text-center rounded-xl border-2 border-dashed cursor-pointer transition-colors border-primary-light bg-primary-light/5"
                                } else {
                                    "flex flex-col gap-2 justify-center items-center p-6 text-center rounded-xl border-2 border-dashed cursor-pointer transition-colors border-stone-300 dark:border-stone-600 hover:border-primary-light hover:bg-primary-light/5"
                                }
                            }
                            on:click=move |_| {
                                #[cfg(feature = "hydrate")]
                                if let Some(input) = file_input_ref.get() {
                                    let input_el: &web_sys::HtmlElement = input.as_ref();
                                    input_el.click();
                                }
                            }
                            on:drop=on_drop
                            on:dragover=on_dragover
                            on:dragleave=on_dragleave
                        >
                            <div class="text-2xl text-stone-400">"\u{1F4F7}"</div>
                            <div class="text-sm text-stone-500 dark:text-stone-400">
                                {move || if is_processing.get() { "Processing..." } else { "Tap to add photo, or drag & drop" }}
                            </div>
                            <input
                                node_ref=file_input_ref
                                type="file"
                                accept="image/jpeg,image/png,image/webp"
                                capture="environment"
                                class="hidden"
                                on:change=on_file_change
                            />
                        </div>
                        {move || error_msg.get().map(|msg| {
                            view! { <p class="mt-1 text-xs text-danger">{msg}</p> }
                        })}
                    }.into_any()
                }
            }}
        </div>
    }.into_any()
}

/// Resize an image from a blob URL using canvas, returning a JPEG data URL.
#[cfg(feature = "hydrate")]
async fn resize_to_data_url(blob_url: &str) -> Result<String, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;

    // Create an <img> and wait for it to load
    let img: web_sys::HtmlImageElement = document
        .create_element("img")
        .map_err(|_| "create img failed")?
        .dyn_into()
        .map_err(|_| "cast to img failed")?;

    // Use a Promise to await the image load event
    let img_for_promise = img.clone();
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        img_for_promise.set_onload(Some(resolve.unchecked_ref()));
        img_for_promise.set_onerror(Some(reject.unchecked_ref()));
    });
    img.set_src(blob_url);
    JsFuture::from(promise).await.map_err(|_| "image load failed")?;

    let orig_w = img.natural_width();
    let orig_h = img.natural_height();
    if orig_w == 0 || orig_h == 0 {
        return Err("Image has zero dimensions".to_string());
    }

    // Calculate target dimensions, preserving aspect ratio
    let max_dim = MAX_IMAGE_DIMENSION;
    let (target_w, target_h) = if orig_w <= max_dim && orig_h <= max_dim {
        (orig_w, orig_h)
    } else if orig_w >= orig_h {
        let ratio = max_dim as f64 / orig_w as f64;
        (max_dim, (orig_h as f64 * ratio).round() as u32)
    } else {
        let ratio = max_dim as f64 / orig_h as f64;
        ((orig_w as f64 * ratio).round() as u32, max_dim)
    };

    // Draw onto a canvas at the target size
    let canvas: web_sys::HtmlCanvasElement = document
        .create_element("canvas")
        .map_err(|_| "create canvas failed")?
        .dyn_into()
        .map_err(|_| "cast to canvas failed")?;
    canvas.set_width(target_w);
    canvas.set_height(target_h);

    let ctx: web_sys::CanvasRenderingContext2d = canvas
        .get_context("2d")
        .map_err(|_| "get context failed")?
        .ok_or("no 2d context")?
        .dyn_into()
        .map_err(|_| "cast context failed")?;

    ctx.draw_image_with_html_image_element_and_dw_and_dh(
        &img,
        0.0,
        0.0,
        target_w as f64,
        target_h as f64,
    )
    .map_err(|_| "draw_image failed")?;

    // Export as JPEG data URL
    canvas
        .to_data_url_with_type("image/jpeg")
        .map_err(|_| "toDataURL failed".to_string())
}
