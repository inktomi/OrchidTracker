use leptos::prelude::*;

#[component]
pub fn PhotoCapture(
    on_photo_ready: impl Fn(String) + 'static + Copy + Send + Sync,
    #[prop(optional)] on_clear: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
) -> impl IntoView {
    let (uploaded_filename, set_uploaded_filename) = signal(Option::<String>::None);
    let (is_uploading, set_is_uploading) = signal(false);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (is_dragging, set_is_dragging) = signal(false);
    let file_input_ref = NodeRef::<leptos::html::Input>::new();
    let on_clear_stored = StoredValue::new(on_clear);
    // These are only used in #[cfg(feature = "hydrate")] blocks
    let _ = &on_photo_ready;
    let _ = &set_is_uploading;
    let _ = &set_error_msg;

    #[cfg(feature = "hydrate")]
    let do_upload = move |file: web_sys::File| {
        set_is_uploading.set(true);
        set_error_msg.set(None);

        leptos::task::spawn_local(async move {
            use wasm_bindgen::JsCast;
            use wasm_bindgen_futures::JsFuture;

            let form_data = match web_sys::FormData::new() {
                Ok(fd) => fd,
                Err(_) => {
                    set_error_msg.set(Some("Failed to create form data".into()));
                    set_is_uploading.set(false);
                    return;
                }
            };
            let _ = form_data.append_with_blob("image", &file);

            let opts = web_sys::RequestInit::new();
            opts.set_method("POST");
            opts.set_body(&form_data.into());

            let request = match web_sys::Request::new_with_str_and_init("/api/images/upload", &opts) {
                Ok(r) => r,
                Err(_) => {
                    set_error_msg.set(Some("Failed to create request".into()));
                    set_is_uploading.set(false);
                    return;
                }
            };

            let window = web_sys::window().unwrap();
            let resp_value = match JsFuture::from(window.fetch_with_request(&request)).await {
                Ok(r) => r,
                Err(_) => {
                    set_error_msg.set(Some("Upload failed".into()));
                    set_is_uploading.set(false);
                    return;
                }
            };

            let resp: web_sys::Response = match resp_value.dyn_into() {
                Ok(r) => r,
                Err(_) => {
                    set_error_msg.set(Some("Invalid response".into()));
                    set_is_uploading.set(false);
                    return;
                }
            };

            if !resp.ok() {
                set_error_msg.set(Some(format!("Upload error: {}", resp.status())));
                set_is_uploading.set(false);
                return;
            }

            let json = match resp.json() {
                Ok(p) => match JsFuture::from(p).await {
                    Ok(j) => j,
                    Err(_) => {
                        set_error_msg.set(Some("Failed to parse response".into()));
                        set_is_uploading.set(false);
                        return;
                    }
                },
                Err(_) => {
                    set_error_msg.set(Some("Failed to read response".into()));
                    set_is_uploading.set(false);
                    return;
                }
            };

            let filename = js_sys::Reflect::get(&json, &"filename".into())
                .ok()
                .and_then(|v| v.as_string());

            match filename {
                Some(fname) => {
                    set_uploaded_filename.set(Some(fname.clone()));
                    on_photo_ready(fname);
                }
                None => {
                    set_error_msg.set(Some("No filename in response".into()));
                }
            }
            set_is_uploading.set(false);
        });
    };

    let on_file_change = move |_ev: leptos::ev::Event| {
        #[cfg(feature = "hydrate")]
        {
            if let Some(input) = file_input_ref.get() {
                let input_el: &web_sys::HtmlInputElement = input.as_ref();
                if let Some(files) = input_el.files() {
                    if let Some(file) = files.get(0) {
                        do_upload(file);
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
                        do_upload(file);
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
        set_uploaded_filename.set(None);
        on_clear_stored.with_value(|oc| {
            if let Some(cb) = oc {
                cb();
            }
        });
    };

    view! {
        <div>
            {move || {
                if let Some(filename) = uploaded_filename.get() {
                    view! {
                        <div class="inline-block relative">
                            <img
                                src=format!("/images/{}", filename)
                                class="block max-w-full rounded-lg border max-h-[200px] border-stone-200 dark:border-stone-700"
                                alt="Uploaded photo"
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
                                {move || if is_uploading.get() { "Uploading..." } else { "Tap to add photo, or drag & drop" }}
                            </div>
                            <input
                                node_ref=file_input_ref
                                type="file"
                                accept="image/jpeg,image/png"
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
