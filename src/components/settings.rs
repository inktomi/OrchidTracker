use leptos::prelude::*;
use crate::orchid::GrowingZone;
use super::{MODAL_OVERLAY, MODAL_CONTENT, MODAL_HEADER, BTN_PRIMARY, BTN_CLOSE, BTN_SECONDARY, BTN_DANGER};

const INPUT_SM: &str = "w-full px-3 py-2 text-sm bg-white/80 border border-stone-300/50 rounded-lg outline-none transition-all duration-200 placeholder:text-stone-400 focus:bg-white focus:border-primary/40 focus:ring-2 focus:ring-primary/10 dark:bg-stone-800/80 dark:border-stone-600/50 dark:placeholder:text-stone-500 dark:focus:bg-stone-800 dark:focus:border-primary-light/40 dark:focus:ring-primary-light/10";
const LABEL_SM: &str = "block mb-1 text-xs font-semibold tracking-wider uppercase text-stone-400 dark:text-stone-500";

#[component]
pub fn SettingsModal(
    zones: Vec<GrowingZone>,
    on_close: impl Fn(String) + 'static + Copy + Send + Sync,
    on_zones_changed: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let (temp_unit, set_temp_unit) = signal("C".to_string());

    // Zone management state
    let (show_add_zone, set_show_add_zone) = signal(false);
    let (add_name, set_add_name) = signal(String::new());
    let (add_light, set_add_light) = signal("Medium".to_string());
    let (add_location, set_add_location) = signal("Indoor".to_string());
    let (add_temp, set_add_temp) = signal(String::new());
    let (add_humidity, set_add_humidity) = signal(String::new());
    let (add_desc, set_add_desc) = signal(String::new());

    let (is_zone_saving, set_is_zone_saving) = signal(false);
    let (local_zones, set_local_zones) = signal(zones);

    let reset_add_form = move || {
        set_add_name.set(String::new());
        set_add_light.set("Medium".to_string());
        set_add_location.set("Indoor".to_string());
        set_add_temp.set(String::new());
        set_add_humidity.set(String::new());
        set_add_desc.set(String::new());
        set_show_add_zone.set(false);
    };

    let add_zone = move |_| {
        let name = add_name.get();
        if name.is_empty() { return; }
        set_is_zone_saving.set(true);

        let light = add_light.get();
        let location = add_location.get();
        let temp = add_temp.get();
        let humidity = add_humidity.get();
        let desc = add_desc.get();
        let sort_order = local_zones.get().len() as i32;

        leptos::task::spawn_local(async move {
            match crate::server_fns::zones::create_zone(
                name, light, location, temp, humidity, desc, sort_order,
            ).await {
                Ok(zone) => {
                    set_local_zones.update(|z| z.push(zone));
                    on_zones_changed();
                }
                Err(e) => {
                    log::error!("Failed to create zone: {}", e);
                }
            }
            set_is_zone_saving.set(false);
            reset_add_form();
        });
    };

    let delete_zone = move |id: String| {
        set_is_zone_saving.set(true);
        let zone_id = id.clone();
        leptos::task::spawn_local(async move {
            match crate::server_fns::zones::delete_zone(zone_id.clone()).await {
                Ok(()) => {
                    set_local_zones.update(|z| z.retain(|zone| zone.id != zone_id));
                    on_zones_changed();
                }
                Err(e) => {
                    log::error!("Failed to delete zone: {}", e);
                }
            }
            set_is_zone_saving.set(false);
        });
    };

    view! {
        <div class=MODAL_OVERLAY>
            <div class=MODAL_CONTENT>
                <div class=MODAL_HEADER>
                    <h2 class="m-0">"Settings"</h2>
                    <button class=BTN_CLOSE on:click=move |_| on_close(temp_unit.get_untracked())>"Close"</button>
                </div>
                <div>
                    <div class="mb-4">
                        <label>"Temperature Unit:"</label>
                        <select
                            on:change=move |ev| set_temp_unit.set(event_target_value(&ev))
                            prop:value=temp_unit
                        >
                            <option value="C">"Celsius (C)"</option>
                            <option value="F">"Fahrenheit (F)"</option>
                        </select>
                    </div>

                    <hr class="my-6 border-stone-200 dark:border-stone-700" />

                    // Growing Zones section
                    <div class="mb-6">
                        <h3 class="mb-4 text-sm font-semibold tracking-wider uppercase text-stone-500 dark:text-stone-400">"Growing Zones"</h3>

                        <div class="flex flex-col gap-2 mb-4">
                            <For
                                each=move || local_zones.get()
                                key=|zone| zone.id.clone()
                                children=move |zone| {
                                    let zone_id = zone.id.clone();
                                    let light_class = match zone.light_level {
                                        crate::orchid::LightRequirement::High => "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-300",
                                        crate::orchid::LightRequirement::Low => "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300",
                                        _ => "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300",
                                    };
                                    let loc_class = match zone.location_type {
                                        crate::orchid::LocationType::Outdoor => "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-sky-100 text-sky-700 dark:bg-sky-900/30 dark:text-sky-300",
                                        _ => "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-stone-100 text-stone-600 dark:bg-stone-800 dark:text-stone-400",
                                    };

                                    view! {
                                        <div class="flex justify-between items-center p-3 rounded-xl border bg-secondary/30 border-stone-200/60 dark:border-stone-700">
                                            <div class="flex flex-col gap-1">
                                                <span class="text-sm font-medium text-stone-700 dark:text-stone-300">{zone.name}</span>
                                                <div class="flex gap-2">
                                                    <span class=light_class>{zone.light_level.to_string()}</span>
                                                    <span class=loc_class>{zone.location_type.to_string()}</span>
                                                </div>
                                            </div>
                                            <button
                                                class=BTN_DANGER
                                                disabled=move || is_zone_saving.get()
                                                on:click=move |_| delete_zone(zone_id.clone())
                                            >"Delete"</button>
                                        </div>
                                    }
                                }
                            />
                        </div>

                        // Add zone form
                        {move || if show_add_zone.get() {
                            view! {
                                <div class="p-4 mb-4 rounded-xl border bg-secondary/30 border-stone-200/60 dark:border-stone-700">
                                    <div class="mb-3">
                                        <label class=LABEL_SM>"Name"</label>
                                        <input type="text" class=INPUT_SM
                                            placeholder="e.g. Kitchen Windowsill"
                                            prop:value=add_name
                                            on:input=move |ev| set_add_name.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="flex gap-3 mb-3">
                                        <div class="flex-1">
                                            <label class=LABEL_SM>"Light"</label>
                                            <select class=INPUT_SM
                                                prop:value=add_light
                                                on:change=move |ev| set_add_light.set(event_target_value(&ev))
                                            >
                                                <option value="Low">"Low"</option>
                                                <option value="Medium">"Medium"</option>
                                                <option value="High">"High"</option>
                                            </select>
                                        </div>
                                        <div class="flex-1">
                                            <label class=LABEL_SM>"Location"</label>
                                            <select class=INPUT_SM
                                                prop:value=add_location
                                                on:change=move |ev| set_add_location.set(event_target_value(&ev))
                                            >
                                                <option value="Indoor">"Indoor"</option>
                                                <option value="Outdoor">"Outdoor"</option>
                                            </select>
                                        </div>
                                    </div>
                                    <div class="flex gap-3 mb-3">
                                        <div class="flex-1">
                                            <label class=LABEL_SM>"Temp Range"</label>
                                            <input type="text" class=INPUT_SM
                                                placeholder="e.g. 18-28C"
                                                prop:value=add_temp
                                                on:input=move |ev| set_add_temp.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div class="flex-1">
                                            <label class=LABEL_SM>"Humidity"</label>
                                            <input type="text" class=INPUT_SM
                                                placeholder="e.g. 50-70%"
                                                prop:value=add_humidity
                                                on:input=move |ev| set_add_humidity.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>
                                    <div class="mb-3">
                                        <label class=LABEL_SM>"Description"</label>
                                        <input type="text" class=INPUT_SM
                                            placeholder="Optional notes"
                                            prop:value=add_desc
                                            on:input=move |ev| set_add_desc.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="flex gap-2">
                                        <button class=BTN_PRIMARY
                                            disabled=move || is_zone_saving.get()
                                            on:click=add_zone
                                        >"Add"</button>
                                        <button class=BTN_SECONDARY
                                            on:click=move |_| reset_add_form()
                                        >"Cancel"</button>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <button
                                    class="flex gap-2 justify-center items-center py-2 mb-4 w-full text-sm font-medium rounded-xl border border-dashed transition-colors cursor-pointer text-stone-400 border-stone-300 dark:border-stone-600 hover:text-primary hover:border-primary/40"
                                    on:click=move |_| set_show_add_zone.set(true)
                                >
                                    "+  Add Zone"
                                </button>
                            }.into_any()
                        }}
                    </div>

                    <p class="p-3 mb-4 text-xs leading-relaxed rounded-lg text-stone-500 bg-secondary dark:text-stone-400">
                        "API keys and sync settings are managed server-side. Contact your administrator to update them."
                    </p>

                    <div class="mt-6">
                        <button class=BTN_PRIMARY on:click=move |_| on_close(temp_unit.get_untracked())>"Save Settings"</button>
                    </div>
                </div>
            </div>
        </div>
    }
}
