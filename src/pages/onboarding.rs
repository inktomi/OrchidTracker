use leptos::prelude::*;
use crate::server_fns::auth::get_current_user;
use crate::server_fns::zones::create_zone;

const INPUT_CLASS: &str = "w-full px-4 py-3 text-sm bg-white/80 border border-stone-300/50 rounded-xl outline-none transition-all duration-200 placeholder:text-stone-400 focus:bg-white focus:border-primary/40 focus:ring-2 focus:ring-primary/10 dark:bg-stone-800/80 dark:border-stone-600/50 dark:placeholder:text-stone-500 dark:focus:bg-stone-800 dark:focus:border-primary-light/40 dark:focus:ring-primary-light/10";
const LABEL_CLASS: &str = "block mb-2 text-xs font-semibold tracking-widest uppercase text-stone-400 dark:text-stone-500";

#[derive(Clone, Debug, PartialEq)]
struct PendingZone {
    temp_id: String,
    name: String,
    light_level: String,
    location_type: String,
    temperature_range: String,
    humidity: String,
    description: String,
}

fn gen_id() -> String {
    #[cfg(feature = "hydrate")]
    {
        let val = js_sys::Math::random();
        format!("zone_{}", (val * 1_000_000.0) as u64)
    }
    #[cfg(not(feature = "hydrate"))]
    {
        format!("zone_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0))
    }
}

fn light_badge_class(level: &str) -> &'static str {
    match level {
        "High" => "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-300",
        "Low" => "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300",
        _ => "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300",
    }
}

fn location_badge_class(loc: &str) -> &'static str {
    match loc {
        "Outdoor" => "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-sky-100 text-sky-700 dark:bg-sky-900/30 dark:text-sky-300",
        _ => "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-stone-100 text-stone-600 dark:bg-stone-800 dark:text-stone-400",
    }
}

fn border_accent(level: &str) -> &'static str {
    match level {
        "High" => "border-l-amber-400",
        "Low" => "border-l-blue-400",
        _ => "border-l-emerald-400",
    }
}

#[component]
pub fn OnboardingPage() -> impl IntoView {
    let user = Resource::new(|| (), |_| get_current_user());

    let (step, set_step) = signal(0u32);
    let (zones, set_zones) = signal::<Vec<PendingZone>>(Vec::new());
    let (is_saving, set_is_saving) = signal(false);

    let finish_setup = move |_: leptos::ev::MouseEvent| {
        set_is_saving.set(true);
        let current_zones = zones.get_untracked();

        leptos::task::spawn_local(async move {
            for (i, zone) in current_zones.iter().enumerate() {
                let _ = create_zone(
                    zone.name.clone(),
                    zone.light_level.clone(),
                    zone.location_type.clone(),
                    zone.temperature_range.clone(),
                    zone.humidity.clone(),
                    zone.description.clone(),
                    i as i32,
                ).await;
            }
            #[cfg(feature = "hydrate")]
            {
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href("/");
                }
            }
        });
    };

    view! {
        // Auth check
        <Suspense fallback=move || view! { <p class="p-8 text-center text-stone-500">"Loading..."</p> }>
            {move || {
                user.get().map(|result| match result {
                    Ok(Some(_)) => view! { <div></div> }.into_any(),
                    _ => {
                        #[cfg(feature = "ssr")]
                        leptos_axum::redirect("/login");
                        #[cfg(feature = "hydrate")]
                        {
                            if let Some(window) = web_sys::window() {
                                let _ = window.location().set_href("/login");
                            }
                        }
                        view! { <div></div> }.into_any()
                    }
                })
            }}
        </Suspense>

        <div class="flex min-h-screen bg-cream">
            // Left panel — botanical (hidden on mobile)
            <LeftPanel step=step />

            // Right panel — wizard content
            <div class="flex flex-col justify-center items-center py-12 px-6 w-full sm:px-12 lg:w-1/2 xl:w-2/5">
                <div class="w-full max-w-lg">
                    // Mobile brand
                    <div class="flex gap-2 justify-center items-center mb-8 lg:hidden">
                        <div class="flex justify-center items-center w-8 h-8 text-sm rounded-lg bg-primary">"🌿"</div>
                        <span class="text-sm font-semibold tracking-widest uppercase text-primary">"Orchid Tracker"</span>
                    </div>

                    // Mobile step indicator
                    <div class="flex gap-2 justify-center items-center mb-8 lg:hidden">
                        {(0..3).map(|i| {
                            let is_active = move || step.get() >= i;
                            view! {
                                <div class=move || {
                                    if is_active() {
                                        "w-8 h-1 rounded-full bg-primary transition-all duration-300"
                                    } else {
                                        "w-8 h-1 rounded-full bg-stone-200 dark:bg-stone-700 transition-all duration-300"
                                    }
                                }></div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>

                    // Step 0: Welcome
                    {move || (step.get() == 0).then(|| view! {
                        <StepWelcome on_continue=move || set_step.set(1) />
                    })}

                    // Step 1: Zone Builder
                    {move || (step.get() == 1).then(|| view! {
                        <StepZoneBuilder
                            zones=zones
                            set_zones=set_zones
                            on_back=move || set_step.set(0)
                            on_continue=move || set_step.set(2)
                        />
                    })}

                    // Step 2: Review & Finish
                    {move || (step.get() == 2).then(|| view! {
                        <StepReview
                            zones=zones
                            is_saving=is_saving
                            on_back=move || set_step.set(1)
                            on_finish=finish_setup
                        />
                    })}
                </div>
            </div>
        </div>
    }
}

/// Left decorative panel with step-dependent messaging
#[component]
fn LeftPanel(step: ReadSignal<u32>) -> impl IntoView {
    // .into_any() erases the concrete view type so the parent's type tree stays shallow
    view! {
        <div class="hidden overflow-hidden relative lg:flex lg:w-1/2 xl:w-3/5 bg-primary">
            <div class="absolute inset-0 bg-gradient-to-br from-primary via-primary-dark to-primary-dark"></div>
            <div class="absolute inset-0 auth-glow-green"></div>
            <div class="absolute inset-0 auth-glow-gold"></div>
            <div class="absolute inset-0 auth-grid opacity-[0.04]"></div>
            <div class="flex relative z-10 flex-col justify-between p-12 xl:p-16">
                <div>
                    <div class="flex gap-3 items-center mb-2">
                        <div class="flex justify-center items-center w-10 h-10 text-lg rounded-xl border bg-white/10 border-white/20">"🌿"</div>
                        <span class="text-sm font-semibold tracking-widest uppercase text-white/70">"Orchid Tracker"</span>
                    </div>
                </div>

                <div class="max-w-lg">
                    {move || match step.get() {
                        0 => view! {
                            <h1 class="mb-6 text-5xl leading-tight text-white xl:text-6xl">"Welcome to your growing space"</h1>
                            <p class="text-lg leading-relaxed text-white/60">
                                "Before adding orchids, let's define where they'll live. Each zone represents a physical location with its own light, temperature, and humidity conditions."
                            </p>
                        }.into_any(),
                        1 => view! {
                            <h1 class="mb-6 text-5xl leading-tight text-white xl:text-6xl">"Build your zones"</h1>
                            <p class="text-lg leading-relaxed text-white/60">
                                "Start with a template or create custom zones. Each zone captures the environmental conditions of a growing location."
                            </p>
                        }.into_any(),
                        _ => view! {
                            <h1 class="mb-6 text-5xl leading-tight text-white xl:text-6xl">"Looking good!"</h1>
                            <p class="text-lg leading-relaxed text-white/60">
                                "Review your growing zones below. You can always add, edit, or remove zones later from Settings."
                            </p>
                        }.into_any(),
                    }}
                </div>

                // Step indicator
                <div class="flex gap-3 items-center pt-8 border-t border-white/10">
                    {(0..3).map(|i| {
                        let is_active = move || step.get() >= i;
                        view! {
                            <div class=move || {
                                if is_active() {
                                    "w-12 h-1.5 rounded-full bg-accent-light transition-all duration-300"
                                } else {
                                    "w-12 h-1.5 rounded-full bg-white/20 transition-all duration-300"
                                }
                            }></div>
                        }
                    }).collect::<Vec<_>>()}
                    <span class="ml-2 text-xs text-white/40">{move || format!("Step {} of 3", step.get() + 1)}</span>
                </div>
            </div>
        </div>
    }.into_any()
}

/// Step 0: Welcome screen
#[component]
fn StepWelcome(
    on_continue: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    view! {
        <div>
            <h2 class="mb-2 text-3xl text-stone-800 dark:text-stone-100">"Set up your growing space"</h2>
            <p class="mb-8 text-sm leading-relaxed text-stone-400 dark:text-stone-500">
                "Zones represent the different places where your orchids grow \u{2014} a shelf, a windowsill, a patio. Each zone tracks light, temperature, and humidity so the app can suggest the best placement for each orchid."
            </p>

            <div class="p-5 mb-8 rounded-xl border bg-secondary/50 border-stone-200/60 dark:border-stone-700/60">
                <div class="flex gap-4 items-start">
                    <div class="flex flex-shrink-0 justify-center items-center w-10 h-10 text-lg rounded-xl bg-primary/10">"💡"</div>
                    <div>
                        <p class="text-sm font-medium text-stone-700 dark:text-stone-300">"Quick start available"</p>
                        <p class="mt-1 text-xs text-stone-400">"Choose a template in the next step, or create zones from scratch."</p>
                    </div>
                </div>
            </div>

            <button
                class="flex gap-2 justify-center items-center py-3 w-full text-sm font-semibold text-white rounded-xl border-none transition-all duration-200 cursor-pointer hover:shadow-lg bg-primary hover:bg-primary-dark hover:shadow-primary/20 active:scale-[0.98]"
                on:click=move |_| on_continue()
            >
                "Get Started"
            </button>
        </div>
    }.into_any()
}

/// Step 1: Zone builder with templates, zone list, and collapsible custom form
#[component]
fn StepZoneBuilder(
    zones: ReadSignal<Vec<PendingZone>>,
    set_zones: WriteSignal<Vec<PendingZone>>,
    on_back: impl Fn() + 'static + Copy + Send + Sync,
    on_continue: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let (show_add_form, set_show_add_form) = signal(false);
    let (add_name, set_add_name) = signal(String::new());
    let (add_light, set_add_light) = signal("Medium".to_string());
    let (add_location, set_add_location) = signal("Indoor".to_string());
    let (add_temp, set_add_temp) = signal(String::new());
    let (add_humidity, set_add_humidity) = signal(String::new());
    let (add_desc, set_add_desc) = signal(String::new());

    let reset_add_form = move || {
        set_add_name.set(String::new());
        set_add_light.set("Medium".to_string());
        set_add_location.set("Indoor".to_string());
        set_add_temp.set(String::new());
        set_add_humidity.set(String::new());
        set_add_desc.set(String::new());
        set_show_add_form.set(false);
    };

    let add_custom_zone = move |_| {
        let name = add_name.get();
        if name.is_empty() { return; }
        set_zones.update(|z| {
            z.push(PendingZone {
                temp_id: gen_id(),
                name,
                light_level: add_light.get(),
                location_type: add_location.get(),
                temperature_range: add_temp.get(),
                humidity: add_humidity.get(),
                description: add_desc.get(),
            });
        });
        reset_add_form();
    };

    let apply_template = move |template: &str| {
        let new_zones: Vec<PendingZone> = match template {
            "indoor_shelves" => vec![
                PendingZone { temp_id: gen_id(), name: "Top Shelf".into(), light_level: "High".into(), location_type: "Indoor".into(), temperature_range: String::new(), humidity: String::new(), description: "Highest light position".into() },
                PendingZone { temp_id: gen_id(), name: "Middle Shelf".into(), light_level: "Medium".into(), location_type: "Indoor".into(), temperature_range: String::new(), humidity: String::new(), description: "Moderate light position".into() },
                PendingZone { temp_id: gen_id(), name: "Bottom Shelf".into(), light_level: "Low".into(), location_type: "Indoor".into(), temperature_range: String::new(), humidity: String::new(), description: "Lower light position".into() },
            ],
            "windowsill" => vec![
                PendingZone { temp_id: gen_id(), name: "Bright Windowsill".into(), light_level: "High".into(), location_type: "Indoor".into(), temperature_range: String::new(), humidity: String::new(), description: "South or west-facing window".into() },
                PendingZone { temp_id: gen_id(), name: "Shaded Windowsill".into(), light_level: "Low".into(), location_type: "Indoor".into(), temperature_range: String::new(), humidity: String::new(), description: "North or east-facing window".into() },
            ],
            "greenhouse" => vec![
                PendingZone { temp_id: gen_id(), name: "Greenhouse".into(), light_level: "High".into(), location_type: "Indoor".into(), temperature_range: "18-32C".into(), humidity: "60-80%".into(), description: "Controlled greenhouse environment".into() },
            ],
            "outdoor" => vec![
                PendingZone { temp_id: gen_id(), name: "Full Sun Area".into(), light_level: "High".into(), location_type: "Outdoor".into(), temperature_range: String::new(), humidity: String::new(), description: "Direct sunlight, open garden".into() },
                PendingZone { temp_id: gen_id(), name: "Partial Shade".into(), light_level: "Medium".into(), location_type: "Outdoor".into(), temperature_range: String::new(), humidity: String::new(), description: "Under tree canopy or covered patio".into() },
            ],
            _ => vec![],
        };
        set_zones.update(|z| z.extend(new_zones));
    };

    let remove_zone = move |id: String| {
        set_zones.update(|z| z.retain(|zone| zone.temp_id != id));
    };

    let can_continue = Memo::new(move |_| !zones.get().is_empty());

    view! {
        <div>
            <h2 class="mb-2 text-3xl text-stone-800 dark:text-stone-100">"Add growing zones"</h2>
            <p class="mb-6 text-sm text-stone-400 dark:text-stone-500">"Start with a template or add custom zones."</p>

            // Template cards
            <TemplateCards apply_template=apply_template />

            // Zone list
            {move || {
                let current_zones = zones.get();
                if current_zones.is_empty() {
                    None
                } else {
                    Some(view! {
                        <div class="mb-6">
                            <h3 class="mb-3 text-xs font-semibold tracking-widest uppercase text-stone-400">"Your Zones"</h3>
                            <div class="flex flex-col gap-2">
                                {current_zones.iter().map(|zone| {
                                    let zone_id = zone.temp_id.clone();
                                    let border = border_accent(&zone.light_level);
                                    let light_class = light_badge_class(&zone.light_level);
                                    let loc_class = location_badge_class(&zone.location_type);
                                    view! {
                                        <div class=format!("flex justify-between items-center p-3 rounded-xl border-l-4 bg-surface border border-stone-200/60 dark:border-stone-700 {}", border)>
                                            <div class="flex flex-col gap-1">
                                                <span class="text-sm font-medium text-stone-700 dark:text-stone-300">{zone.name.clone()}</span>
                                                <div class="flex gap-2">
                                                    <span class=light_class>{zone.light_level.clone()}</span>
                                                    <span class=loc_class>{zone.location_type.clone()}</span>
                                                </div>
                                            </div>
                                            <button
                                                class="py-1 px-2 text-xs rounded-lg border-none transition-colors cursor-pointer text-stone-400 bg-stone-100 dark:bg-stone-800 hover:bg-danger/10 hover:text-danger"
                                                on:click=move |_| remove_zone(zone_id.clone())
                                            >"Remove"</button>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    })
                }
            }}

            // Collapsible custom zone section
            <CustomZoneForm
                show=show_add_form
                set_show=set_show_add_form
                add_name=add_name set_add_name=set_add_name
                add_light=add_light set_add_light=set_add_light
                add_location=add_location set_add_location=set_add_location
                add_temp=add_temp set_add_temp=set_add_temp
                add_humidity=add_humidity set_add_humidity=set_add_humidity
                add_desc=add_desc set_add_desc=set_add_desc
                on_add=add_custom_zone
                on_cancel=move || reset_add_form()
            />

            // Navigation
            <div class="flex gap-3">
                <button
                    class="py-2 px-4 text-sm font-medium rounded-xl border-none transition-colors cursor-pointer text-stone-500 bg-stone-100 dark:bg-stone-800 dark:text-stone-400 dark:hover:bg-stone-700 hover:bg-stone-200"
                    on:click=move |_| on_back()
                >"Back"</button>
                <button
                    class="flex-1 py-3 text-sm font-semibold text-white rounded-xl border-none transition-all duration-200 cursor-pointer hover:shadow-lg disabled:opacity-50 disabled:cursor-not-allowed bg-primary hover:bg-primary-dark hover:shadow-primary/20 active:scale-[0.98]"
                    disabled=move || !can_continue.get()
                    on:click=move |_| on_continue()
                >"Continue"</button>
            </div>
        </div>
    }.into_any()
}

/// Template selection cards — extracted to reduce view nesting depth
#[component]
fn TemplateCards(
    apply_template: impl Fn(&str) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let tpl_btn: &str = "p-4 text-left rounded-xl border transition-all duration-200 cursor-pointer border-stone-200/60 bg-surface dark:border-stone-700 dark:hover:border-primary-light/40 hover:border-primary/40 hover:bg-primary/5";
    view! {
        <div class="grid grid-cols-2 gap-3 mb-8">
            <button class=tpl_btn on:click=move |_| apply_template("indoor_shelves")>
                <div class="mb-2 text-2xl">"🏠"</div>
                <div class="text-sm font-medium text-stone-700 dark:text-stone-300">"Indoor Shelves"</div>
                <div class="mt-1 text-xs text-stone-400">"3 shelves: High, Med, Low"</div>
            </button>
            <button class=tpl_btn on:click=move |_| apply_template("windowsill")>
                <div class="mb-2 text-2xl">"🪟"</div>
                <div class="text-sm font-medium text-stone-700 dark:text-stone-300">"Windowsill"</div>
                <div class="mt-1 text-xs text-stone-400">"Bright + Shaded sills"</div>
            </button>
            <button class=tpl_btn on:click=move |_| apply_template("greenhouse")>
                <div class="mb-2 text-2xl">"🌡️"</div>
                <div class="text-sm font-medium text-stone-700 dark:text-stone-300">"Greenhouse"</div>
                <div class="mt-1 text-xs text-stone-400">"Controlled environment"</div>
            </button>
            <button class=tpl_btn on:click=move |_| apply_template("outdoor")>
                <div class="mb-2 text-2xl">"☀️"</div>
                <div class="text-sm font-medium text-stone-700 dark:text-stone-300">"Outdoor Garden"</div>
                <div class="mt-1 text-xs text-stone-400">"Full sun + Partial shade"</div>
            </button>
        </div>
    }.into_any()
}

/// Collapsible custom zone form with sprout animation
#[component]
fn CustomZoneForm(
    show: ReadSignal<bool>,
    set_show: WriteSignal<bool>,
    add_name: ReadSignal<String>,
    set_add_name: WriteSignal<String>,
    add_light: ReadSignal<String>,
    set_add_light: WriteSignal<String>,
    add_location: ReadSignal<String>,
    set_add_location: WriteSignal<String>,
    add_temp: ReadSignal<String>,
    set_add_temp: WriteSignal<String>,
    add_humidity: ReadSignal<String>,
    set_add_humidity: WriteSignal<String>,
    add_desc: ReadSignal<String>,
    set_add_desc: WriteSignal<String>,
    on_add: impl Fn(leptos::ev::MouseEvent) + 'static + Copy + Send + Sync,
    on_cancel: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    view! {
        <div class="overflow-hidden mb-6 rounded-xl border transition-colors duration-300"
            class=(
                "border-primary/30 bg-primary/[0.03] dark:border-primary-light/20 dark:bg-primary-light/[0.03]",
                move || show.get()
            )
            class=(
                "border-stone-200/60 dark:border-stone-700 border-dashed",
                move || !show.get()
            )
        >
            // Toggle trigger — always visible
            <button
                class="flex gap-3 justify-between items-center py-3 px-4 w-full text-sm font-medium text-left bg-transparent border-none transition-colors duration-200 cursor-pointer"
                class=(
                    "text-primary dark:text-primary-light",
                    move || show.get()
                )
                class=(
                    "text-stone-400 hover:text-primary dark:hover:text-primary-light",
                    move || !show.get()
                )
                on:click=move |_| set_show.update(|v| *v = !*v)
            >
                <div class="flex gap-2.5 items-center">
                    <span
                        class="text-lg"
                        class=("zone-sprout-open", move || show.get())
                        class=("zone-sprout", true)
                    >"🌱"</span>
                    <span>"Add Custom Zone"</span>
                </div>
                <span
                    class="text-xs opacity-60 zone-chevron"
                    class=("zone-chevron-open", move || show.get())
                >"▼"</span>
            </button>

            // Collapsible body — always in DOM, height animated via CSS grid
            <div
                class="zone-collapse"
                class=("zone-collapse-open", move || show.get())
            >
                <div>
                    <div class="px-4 pt-1 pb-4">
                        <div class="mb-3 zone-field">
                            <label class=LABEL_CLASS>"Name"</label>
                            <input type="text" class=INPUT_CLASS
                                placeholder="e.g. Kitchen Windowsill"
                                prop:value=add_name
                                on:input=move |ev| set_add_name.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="flex gap-3 mb-3 zone-field">
                            <div class="flex-1">
                                <label class=LABEL_CLASS>"Light Level"</label>
                                <select class=INPUT_CLASS
                                    prop:value=add_light
                                    on:change=move |ev| set_add_light.set(event_target_value(&ev))
                                >
                                    <option value="Low">"Low"</option>
                                    <option value="Medium">"Medium"</option>
                                    <option value="High">"High"</option>
                                </select>
                            </div>
                            <div class="flex-1">
                                <label class=LABEL_CLASS>"Location"</label>
                                <select class=INPUT_CLASS
                                    prop:value=add_location
                                    on:change=move |ev| set_add_location.set(event_target_value(&ev))
                                >
                                    <option value="Indoor">"Indoor"</option>
                                    <option value="Outdoor">"Outdoor"</option>
                                </select>
                            </div>
                        </div>
                        <div class="flex gap-3 mb-3 zone-field">
                            <div class="flex-1">
                                <label class=LABEL_CLASS>"Temp Range"</label>
                                <input type="text" class=INPUT_CLASS
                                    placeholder="e.g. 18-28C"
                                    prop:value=add_temp
                                    on:input=move |ev| set_add_temp.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="flex-1">
                                <label class=LABEL_CLASS>"Humidity"</label>
                                <input type="text" class=INPUT_CLASS
                                    placeholder="e.g. 50-70%"
                                    prop:value=add_humidity
                                    on:input=move |ev| set_add_humidity.set(event_target_value(&ev))
                                />
                            </div>
                        </div>
                        <div class="mb-4 zone-field">
                            <label class=LABEL_CLASS>"Description"</label>
                            <textarea class=INPUT_CLASS rows="2"
                                placeholder="Optional notes about this zone"
                                prop:value=add_desc
                                on:input=move |ev| set_add_desc.set(event_target_value(&ev))
                            ></textarea>
                        </div>
                        <div class="flex gap-2 zone-field">
                            <button
                                class="flex-1 py-2 text-sm font-semibold text-white rounded-xl border-none transition-colors cursor-pointer bg-primary hover:bg-primary-dark"
                                on:click=on_add
                            >"Add Zone"</button>
                            <button
                                class="py-2 px-4 text-sm font-medium rounded-xl border-none transition-colors cursor-pointer text-stone-500 bg-stone-100 dark:bg-stone-800 dark:text-stone-400 dark:hover:bg-stone-700 hover:bg-stone-200"
                                on:click=move |_| on_cancel()
                            >"Cancel"</button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }.into_any()
}

/// Step 2: Review zones and finish setup
#[component]
fn StepReview(
    zones: ReadSignal<Vec<PendingZone>>,
    is_saving: ReadSignal<bool>,
    on_back: impl Fn() + 'static + Copy + Send + Sync,
    on_finish: impl Fn(leptos::ev::MouseEvent) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let current_zones = zones.get();
    let indoor: Vec<PendingZone> = current_zones.iter().filter(|z| z.location_type == "Indoor").cloned().collect();
    let outdoor: Vec<PendingZone> = current_zones.iter().filter(|z| z.location_type == "Outdoor").cloned().collect();

    view! {
        <div>
            <h2 class="mb-2 text-3xl text-stone-800 dark:text-stone-100">"Review your setup"</h2>
            <p class="mb-6 text-sm text-stone-400 dark:text-stone-500">"You can edit zones later in Settings."</p>

            {if !indoor.is_empty() {
                Some(view! {
                    <ZoneGroupReview label="Indoor Zones" zones=indoor />
                })
            } else {
                None
            }}

            {if !outdoor.is_empty() {
                Some(view! {
                    <ZoneGroupReview label="Outdoor Zones" zones=outdoor />
                })
            } else {
                None
            }}

            <div class="flex gap-3 mt-8">
                <button
                    class="py-2 px-4 text-sm font-medium rounded-xl border-none transition-colors cursor-pointer text-stone-500 bg-stone-100 dark:bg-stone-800 dark:text-stone-400 dark:hover:bg-stone-700 hover:bg-stone-200"
                    on:click=move |_| on_back()
                >"Back"</button>
                <button
                    class="flex-1 py-3 text-sm font-semibold text-white rounded-xl border-none transition-all duration-200 cursor-pointer hover:shadow-lg disabled:opacity-50 disabled:cursor-not-allowed bg-primary hover:bg-primary-dark hover:shadow-primary/20 active:scale-[0.98]"
                    disabled=move || is_saving.get()
                    on:click=on_finish
                >
                    {move || if is_saving.get() { "Saving..." } else { "Finish Setup" }}
                </button>
            </div>
        </div>
    }.into_any()
}

/// Renders a labeled group of zones for the review step
#[component]
fn ZoneGroupReview(label: &'static str, zones: Vec<PendingZone>) -> impl IntoView {
    view! {
        <div class="mb-6">
            <h3 class="mb-3 text-xs font-semibold tracking-widest uppercase text-stone-400">{label}</h3>
            <div class="flex flex-col gap-2">
                {zones.iter().map(|zone| {
                    let border = border_accent(&zone.light_level);
                    let light_class = light_badge_class(&zone.light_level);
                    view! {
                        <div class=format!("p-4 rounded-xl border-l-4 bg-surface border border-stone-200/60 dark:border-stone-700 {}", border)>
                            <div class="flex gap-2 items-center mb-1">
                                <span class="text-sm font-medium text-stone-700 dark:text-stone-300">{zone.name.clone()}</span>
                                <span class=light_class>{format!("{} Light", zone.light_level)}</span>
                            </div>
                            {(!zone.temperature_range.is_empty()).then(|| {
                                view! { <span class="text-xs text-stone-400">{format!("Temp: {}", zone.temperature_range)}</span> }
                            })}
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }.into_any()
}
