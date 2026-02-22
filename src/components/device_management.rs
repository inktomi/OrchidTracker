use leptos::prelude::*;
use crate::orchid::HardwareDevice;
use super::{BTN_PRIMARY, BTN_SECONDARY, BTN_DANGER};

const INPUT_SM: &str = "w-full px-3 py-2 text-sm bg-white/80 border border-stone-300/50 rounded-lg outline-none transition-all duration-200 placeholder:text-stone-400 focus:bg-white focus:border-primary/40 focus:ring-2 focus:ring-primary/10 dark:bg-stone-800/80 dark:border-stone-600/50 dark:placeholder:text-stone-500 dark:focus:bg-stone-800 dark:focus:border-primary-light/40 dark:focus:ring-primary-light/10";
const LABEL_SM: &str = "block mb-1 text-xs font-semibold tracking-wider uppercase text-stone-400 dark:text-stone-500";
const BTN_SM: &str = "py-1.5 px-3 text-xs font-semibold rounded-lg border-none cursor-pointer transition-colors";

/// List of hardware devices with add/edit/delete/test actions.
#[component]
pub fn DeviceList(
    devices: ReadSignal<Vec<HardwareDevice>>,
    set_devices: WriteSignal<Vec<HardwareDevice>>,
) -> impl IntoView {
    let (show_form, set_show_form) = signal(false);
    let (editing_device, set_editing_device) = signal::<Option<HardwareDevice>>(None);

    let on_add = move |_| {
        set_editing_device.set(None);
        set_show_form.set(true);
    };

    let on_edit = move |device: HardwareDevice| {
        set_editing_device.set(Some(device));
        set_show_form.set(true);
    };

    let on_delete = move |device_id: String| {
        leptos::task::spawn_local(async move {
            match crate::server_fns::devices::delete_device(device_id.clone()).await {
                Ok(()) => {
                    set_devices.update(|devs| devs.retain(|d| d.id != device_id));
                }
                Err(e) => {
                    tracing::error!("Failed to delete device: {}", e);
                }
            }
        });
    };

    let on_saved = move |device: HardwareDevice| {
        set_devices.update(|devs| {
            if let Some(existing) = devs.iter_mut().find(|d| d.id == device.id) {
                *existing = device;
            } else {
                devs.push(device);
            }
        });
        set_show_form.set(false);
        set_editing_device.set(None);
    };

    let on_cancel = move || {
        set_show_form.set(false);
        set_editing_device.set(None);
    };

    view! {
        <div>
            <div class="flex flex-col gap-2 mb-4">
                <For
                    each=move || devices.get()
                    key=|d| d.id.clone()
                    children=move |device| {
                        let device_for_edit = device.clone();
                        let device_id_for_delete = device.id.clone();
                        let device_type_badge = match device.device_type.as_str() {
                            "tempest" => ("Tempest", "bg-sky-100 text-sky-700 dark:bg-sky-900/30 dark:text-sky-300"),
                            "ac_infinity" => ("AC Infinity", "bg-violet-100 text-violet-700 dark:bg-violet-900/30 dark:text-violet-300"),
                            _ => ("Unknown", "bg-stone-100 text-stone-600 dark:bg-stone-800 dark:text-stone-400"),
                        };

                        view! {
                            <DeviceCard
                                device=device
                                type_label=device_type_badge.0
                                type_class=device_type_badge.1
                                on_edit=move |_| on_edit(device_for_edit.clone())
                                on_delete=move |_| on_delete(device_id_for_delete.clone())
                            />
                        }
                    }
                />
            </div>

            {move || if show_form.get() {
                view! {
                    <DeviceForm
                        editing=editing_device.get()
                        on_saved=on_saved
                        on_cancel=on_cancel
                    />
                }.into_any()
            } else {
                view! {
                    <button
                        class="flex gap-2 justify-center items-center py-2 w-full text-sm font-medium rounded-xl border border-dashed transition-colors cursor-pointer text-stone-400 border-stone-300 dark:border-stone-600 hover:text-primary hover:border-primary/40"
                        on:click=on_add
                    >
                        "+ Add Device"
                    </button>
                }.into_any()
            }}
        </div>
    }.into_any()
}

/// Individual device card with type badge, edit and delete buttons.
#[component]
fn DeviceCard(
    device: HardwareDevice,
    type_label: &'static str,
    type_class: &'static str,
    on_edit: impl Fn(leptos::ev::MouseEvent) + 'static + Send + Sync,
    on_delete: impl Fn(leptos::ev::MouseEvent) + 'static + Send + Sync,
) -> impl IntoView {
    let (test_result, set_test_result) = signal::<Option<Result<String, String>>>(None);
    let (is_testing, set_is_testing) = signal(false);

    let dev_type = device.device_type.clone();
    let dev_config = device.config.clone();

    let test_connection = move |_| {
        let dt = dev_type.clone();
        let cfg = dev_config.clone();
        set_is_testing.set(true);
        set_test_result.set(None);

        leptos::task::spawn_local(async move {
            match crate::server_fns::devices::test_device(dt, cfg).await {
                Ok(msg) => set_test_result.set(Some(Ok(msg))),
                Err(e) => set_test_result.set(Some(Err(e.to_string()))),
            }
            set_is_testing.set(false);
        });
    };

    view! {
        <div class="rounded-xl border bg-secondary/30 border-stone-200/60 dark:border-stone-700">
            <div class="flex justify-between items-center p-3">
                <div class="flex flex-col gap-1">
                    <span class="text-sm font-medium text-stone-700 dark:text-stone-300">{device.name}</span>
                    <span class=format!("inline-flex self-start py-0.5 px-2 text-xs font-semibold rounded-full {}", type_class)>
                        {type_label}
                    </span>
                </div>
                <div class="flex gap-1.5">
                    <button
                        class=format!("{} text-stone-500 bg-stone-100 hover:bg-stone-200 dark:text-stone-400 dark:bg-stone-800 dark:hover:bg-stone-700", BTN_SM)
                        disabled=move || is_testing.get()
                        on:click=test_connection
                    >{move || if is_testing.get() { "Testing..." } else { "Test" }}</button>
                    <button
                        class=format!("{} text-stone-500 bg-stone-100 hover:bg-stone-200 dark:text-stone-400 dark:bg-stone-800 dark:hover:bg-stone-700", BTN_SM)
                        on:click=on_edit
                    >"Edit"</button>
                    <button
                        class=BTN_DANGER
                        on:click=on_delete
                    >"Delete"</button>
                </div>
            </div>
            {move || test_result.get().map(|result| match result {
                Ok(msg) => view! {
                    <div class="px-3 pb-3">
                        <div class="p-2 text-xs text-emerald-700 bg-emerald-50 rounded-lg dark:text-emerald-300 dark:bg-emerald-900/20">{msg}</div>
                    </div>
                }.into_any(),
                Err(msg) => view! {
                    <div class="px-3 pb-3">
                        <div class="p-2 text-xs text-red-700 bg-red-50 rounded-lg dark:text-red-300 dark:bg-red-900/20">{msg}</div>
                    </div>
                }.into_any(),
            })}
        </div>
    }.into_any()
}

/// Form for creating/editing a hardware device (inline, not modal).
#[component]
fn DeviceForm(
    editing: Option<HardwareDevice>,
    on_saved: impl Fn(HardwareDevice) + 'static + Copy + Send + Sync,
    on_cancel: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let is_edit = editing.is_some();
    let device_id = editing.as_ref().map(|d| d.id.clone()).unwrap_or_default();
    let initial_type = editing.as_ref().map(|d| d.device_type.clone()).unwrap_or_default();

    let (name, set_name) = signal(editing.as_ref().map(|d| d.name.clone()).unwrap_or_default());
    let (device_type, set_device_type) = signal(initial_type.clone());
    let (is_saving, set_is_saving) = signal(false);
    let (error_msg, set_error_msg) = signal::<Option<String>>(None);

    // Parse existing config for field initialization
    let parsed = editing.as_ref()
        .and_then(|d| serde_json::from_str::<serde_json::Value>(&d.config).ok());

    let get_str = |key: &str| -> String {
        parsed.as_ref()
            .and_then(|j| j.get(key))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    };

    // Tempest fields
    let (tempest_station, set_tempest_station) = signal(get_str("station_id"));
    let (tempest_token, set_tempest_token) = signal(get_str("token"));

    // AC Infinity fields
    let (aci_email, set_aci_email) = signal(get_str("email"));
    let (aci_password, set_aci_password) = signal(get_str("password"));
    let (aci_device, set_aci_device) = signal(get_str("device_id"));

    let build_config_json = move || -> String {
        match device_type.get().as_str() {
            "tempest" => serde_json::json!({
                "station_id": tempest_station.get(),
                "token": tempest_token.get(),
            }).to_string(),
            "ac_infinity" => serde_json::json!({
                "email": aci_email.get(),
                "password": aci_password.get(),
                "device_id": aci_device.get(),
            }).to_string(),
            _ => String::new(),
        }
    };

    let dev_id = StoredValue::new(device_id);
    let save = move |_| {
        let n = name.get();
        let dt = device_type.get();
        if n.is_empty() || dt.is_empty() {
            set_error_msg.set(Some("Name and device type are required".into()));
            return;
        }

        set_is_saving.set(true);
        set_error_msg.set(None);
        let config = build_config_json();
        let editing_id = dev_id.get_value();

        leptos::task::spawn_local(async move {
            let result = if is_edit {
                crate::server_fns::devices::update_device(editing_id, n, config).await
            } else {
                crate::server_fns::devices::create_device(n, dt, config).await
            };

            match result {
                Ok(device) => on_saved(device),
                Err(e) => set_error_msg.set(Some(format!("Save failed: {}", e))),
            }
            set_is_saving.set(false);
        });
    };

    view! {
        <div class="p-4 mb-4 rounded-xl border bg-secondary/30 border-stone-200/60 dark:border-stone-700">
            <div class="mb-3">
                <label class=LABEL_SM>"Device Name"</label>
                <input type="text" class=INPUT_SM
                    placeholder="e.g. My Tempest Station"
                    prop:value=name
                    on:input=move |ev| set_name.set(event_target_value(&ev))
                />
            </div>

            {(!is_edit).then(|| view! {
                <div class="mb-3">
                    <label class=LABEL_SM>"Device Type"</label>
                    <select class=INPUT_SM
                        prop:value=device_type
                        on:change=move |ev| set_device_type.set(event_target_value(&ev))
                    >
                        <option value="">"Select type..."</option>
                        <option value="tempest">"Tempest Weather Station"</option>
                        <option value="ac_infinity">"AC Infinity Controller"</option>
                    </select>
                </div>
            })}

            {move || match device_type.get().as_str() {
                "tempest" => view! {
                    <div class="p-3 mb-3 rounded-lg bg-sky-50/50 dark:bg-sky-900/10">
                        <div class="mb-3">
                            <label class=LABEL_SM>"Station ID"</label>
                            <input type="text" class=INPUT_SM
                                placeholder="e.g. 12345"
                                prop:value=tempest_station
                                on:input=move |ev| set_tempest_station.set(event_target_value(&ev))
                            />
                        </div>
                        <div>
                            <label class=LABEL_SM>"API Token"</label>
                            <input type="password" class=INPUT_SM
                                placeholder="Your WeatherFlow API token"
                                prop:value=tempest_token
                                on:input=move |ev| set_tempest_token.set(event_target_value(&ev))
                            />
                        </div>
                    </div>
                }.into_any(),
                "ac_infinity" => view! {
                    <div class="p-3 mb-3 rounded-lg bg-violet-50/50 dark:bg-violet-900/10">
                        <div class="flex gap-3 mb-3">
                            <div class="flex-1">
                                <label class=LABEL_SM>"Email"</label>
                                <input type="email" class=INPUT_SM
                                    placeholder="AC Infinity account email"
                                    prop:value=aci_email
                                    on:input=move |ev| set_aci_email.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="flex-1">
                                <label class=LABEL_SM>"Password"</label>
                                <input type="password" class=INPUT_SM
                                    placeholder="Account password"
                                    prop:value=aci_password
                                    on:input=move |ev| set_aci_password.set(event_target_value(&ev))
                                />
                            </div>
                        </div>
                        <div>
                            <label class=LABEL_SM>"Device ID"</label>
                            <input type="text" class=INPUT_SM
                                placeholder="e.g. ABC123DEF"
                                prop:value=aci_device
                                on:input=move |ev| set_aci_device.set(event_target_value(&ev))
                            />
                        </div>
                    </div>
                }.into_any(),
                _ => view! {
                    <p class="mb-3 text-xs text-stone-400">"Select a device type to configure credentials."</p>
                }.into_any(),
            }}

            {move || error_msg.get().map(|msg| view! {
                <div class="p-2 mb-3 text-xs text-red-700 bg-red-50 rounded-lg dark:text-red-300 dark:bg-red-900/20">{msg}</div>
            })}

            <div class="flex gap-2">
                <button class=BTN_PRIMARY
                    disabled=move || is_saving.get()
                    on:click=save
                >{move || if is_saving.get() { "Saving..." } else if is_edit { "Update" } else { "Create" }}</button>
                <button class=BTN_SECONDARY
                    on:click=move |_| on_cancel()
                >"Cancel"</button>
            </div>
        </div>
    }.into_any()
}
