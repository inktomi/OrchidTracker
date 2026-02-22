use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use crate::orchid::{FitCategory, LightRequirement, Orchid, GrowingZone, ClimateReading};
use super::{MODAL_OVERLAY, BTN_PRIMARY, BTN_GHOST};

const SCANNER_CONTENT: &str = "scanner-bloom bg-stone-900 text-stone-200 p-5 sm:p-8 rounded-2xl w-[95%] sm:w-[90%] max-w-[600px] max-h-[90vh] overflow-y-auto shadow-2xl border border-stone-700/60";
const SCANNER_HEADER: &str = "flex justify-between items-center mb-5 pb-4 border-b border-stone-700";
const SCANNER_CLOSE: &str = "py-2 px-3 text-sm text-stone-400 bg-stone-800 rounded-lg border-none cursor-pointer hover:bg-stone-700 hover:text-stone-200 transition-colors";
const TAB_ACTIVE: &str = "flex-1 py-2.5 text-sm font-semibold text-white rounded-lg border-none cursor-pointer transition-colors bg-primary";
const TAB_INACTIVE: &str = "flex-1 py-2.5 text-sm font-medium rounded-lg border-none cursor-pointer transition-colors text-stone-400 bg-stone-800 hover:text-stone-200 hover:bg-stone-700";
const SEARCH_INPUT: &str = "w-full py-3 px-4 text-sm text-white rounded-xl border outline-none transition-all duration-200 bg-stone-800 border-stone-700 placeholder:text-stone-500 focus:border-primary/50 focus:ring-2 focus:ring-primary/20";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AnalysisResult {
    pub species_name: String,
    pub fit_category: FitCategory,
    pub reason: String,
    pub already_owned: bool,
    pub water_freq: u32,
    pub light_req: LightRequirement,
    pub temp_range: String,
    pub placement_suggestion: String,
    pub conservation_status: Option<String>,
    #[serde(default)]
    pub native_region: Option<String>,
    #[serde(default)]
    pub native_latitude: Option<f64>,
    #[serde(default)]
    pub native_longitude: Option<f64>,
    #[serde(default)]
    pub temp_min: Option<f64>,
    #[serde(default)]
    pub temp_max: Option<f64>,
    #[serde(default)]
    pub humidity_min: Option<f64>,
    #[serde(default)]
    pub humidity_max: Option<f64>,
    #[serde(default)]
    pub rest_start_month: Option<u32>,
    #[serde(default)]
    pub rest_end_month: Option<u32>,
    #[serde(default)]
    pub bloom_start_month: Option<u32>,
    #[serde(default)]
    pub bloom_end_month: Option<u32>,
    #[serde(default)]
    pub rest_water_multiplier: Option<f64>,
    #[serde(default)]
    pub rest_fertilizer_multiplier: Option<f64>,
    #[serde(default)]
    pub active_water_multiplier: Option<f64>,
    #[serde(default)]
    pub active_fertilizer_multiplier: Option<f64>,
}

#[component]
pub fn ScannerModal(
    on_close: impl Fn() + 'static + Copy + Send + Sync,
    on_add_to_collection: impl Fn(AnalysisResult) + 'static + Copy + Send + Sync,
    existing_orchids: Vec<Orchid>,
    climate_readings: Vec<ClimateReading>,
    zones: Vec<GrowingZone>,
) -> impl IntoView {
    // Tab state: true = scan, false = search
    let (scan_mode, set_scan_mode) = signal(true);

    view! {
        <div class=MODAL_OVERLAY>
            <div class=SCANNER_CONTENT>
                // Decorative drifting leaves
                <div class="overflow-hidden absolute inset-0 pointer-events-none">
                    <div class="absolute top-3 right-6 text-lg scanner-leaf-drift opacity-15">{"\u{1F33F}"}</div>
                    <div class="absolute bottom-4 left-5 text-sm opacity-10 scanner-leaf-drift">{"\u{1F343}"}</div>
                    <div class="absolute right-3 top-1/2 text-xs opacity-10 scanner-leaf-drift">{"\u{1FAB4}"}</div>
                </div>

                <div class=SCANNER_HEADER>
                    <div>
                        <h2 class="m-0 text-white">"ID Plant"</h2>
                        <p class="mt-1 mb-0 text-xs text-stone-500">"Scan a tag or search by name"</p>
                    </div>
                    <button class=SCANNER_CLOSE on:click=move |_| on_close()>"Close"</button>
                </div>

                // Tab switcher
                <div class="flex gap-1.5 p-1 mb-5 rounded-xl bg-stone-800/60">
                    <button
                        class=move || if scan_mode.get() { TAB_ACTIVE } else { TAB_INACTIVE }
                        on:click=move |_| set_scan_mode.set(true)
                    >"Scan Tag"</button>
                    <button
                        class=move || if !scan_mode.get() { TAB_ACTIVE } else { TAB_INACTIVE }
                        on:click=move |_| set_scan_mode.set(false)
                    >"Search by Name"</button>
                </div>

                <div class="relative">
                    {move || if scan_mode.get() {
                        view! {
                            <ScanTab
                                on_add_to_collection=on_add_to_collection
                                existing_orchids=existing_orchids.clone()
                                climate_readings=climate_readings.clone()
                                zones=zones.clone()
                            />
                        }.into_any()
                    } else {
                        view! {
                            <SearchTab
                                on_add_to_collection=on_add_to_collection
                                existing_orchids=existing_orchids.clone()
                                climate_readings=climate_readings.clone()
                                zones=zones.clone()
                            />
                        }.into_any()
                    }}
                </div>
            </div>
        </div>
    }.into_any()
}

/// Camera-based tag scanning tab.
#[component]
fn ScanTab(
    on_add_to_collection: impl Fn(AnalysisResult) + 'static + Copy + Send + Sync,
    existing_orchids: Vec<Orchid>,
    climate_readings: Vec<ClimateReading>,
    zones: Vec<GrowingZone>,
) -> impl IntoView {
    let (is_scanning, set_is_scanning) = signal(false);
    let (analysis_result, set_analysis_result) = signal::<Option<AnalysisResult>>(None);
    let (error_msg, set_error_msg) = signal::<Option<String>>(None);

    let video_element: NodeRef<leptos::html::Video> = NodeRef::new();
    let canvas_element: NodeRef<leptos::html::Canvas> = NodeRef::new();

    #[cfg(not(feature = "hydrate"))]
    {
        drop(existing_orchids);
        drop(climate_readings);
        drop(zones);
    }
    #[cfg(feature = "hydrate")]
    let existing_orchids = StoredValue::new(existing_orchids);
    #[cfg(feature = "hydrate")]
    let climate_readings = StoredValue::new(climate_readings);
    #[cfg(feature = "hydrate")]
    let zones = StoredValue::new(zones);

    #[cfg(feature = "hydrate")]
    let (facing_mode, set_facing_mode) = signal("environment".to_string());
    #[cfg(not(feature = "hydrate"))]
    let (_, set_facing_mode) = signal("environment".to_string());

    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsCast;
        let (stream_signal, set_stream_signal) = signal_local::<Option<web_sys::MediaStream>>(None);

        on_cleanup(move || {
            if let Some(stream) = stream_signal.get() {
                let tracks = stream.get_tracks();
                for i in 0..tracks.length() {
                    if let Ok(track) = tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                        track.stop();
                    }
                }
            }
        });

        Effect::new(move |_| {
            let mode = facing_mode.get();

            if let Some(stream) = stream_signal.get_untracked() {
                let tracks = stream.get_tracks();
                for i in 0..tracks.length() {
                    if let Ok(track) = tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                        track.stop();
                    }
                }
            }

            if let Some(video) = video_element.get() {
                let window = web_sys::window().unwrap();
                let navigator = window.navigator();

                leptos::task::spawn_local(async move {
                    if let Ok(media_devices) = navigator.media_devices() {
                        let constraints = web_sys::MediaStreamConstraints::new();
                        let video_constraint = js_sys::Object::new();
                        let _ = js_sys::Reflect::set(&video_constraint, &"facingMode".into(), &mode.into());
                        constraints.set_video(&video_constraint);

                        match media_devices.get_user_media_with_constraints(&constraints) {
                            Ok(promise) => {
                                if let Ok(stream_js) = wasm_bindgen_futures::JsFuture::from(promise).await {
                                    let stream = stream_js.unchecked_into::<web_sys::MediaStream>();
                                    video.set_src_object(Some(&stream));
                                    let _ = video.play();
                                    set_stream_signal.set(Some(stream));
                                }
                            }
                            Err(e) => {
                                tracing::error!("Camera Error: {:?}", e);
                                set_error_msg.set(Some("Camera access denied or not available.".into()));
                            }
                        }
                    }
                });
            }
        });
    }

    let flip_camera = move |_| {
        set_facing_mode.update(|m| *m = if m == "environment" { "user".into() } else { "environment".into() });
    };

    let capture_and_analyze = move |_| {
        set_is_scanning.set(true);
        set_error_msg.set(None);
        set_analysis_result.set(None);

        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;

            let video = video_element.get().expect("Video element missing");
            let canvas = canvas_element.get().expect("Canvas element missing");
            let html_canvas: &web_sys::HtmlCanvasElement = &canvas;

            let context = html_canvas.get_context("2d").unwrap().unwrap().unchecked_into::<web_sys::CanvasRenderingContext2d>();

            let width = video.video_width() as f64;
            let height = video.video_height() as f64;
            html_canvas.set_width(width as u32);
            html_canvas.set_height(height as u32);

            if let Err(e) = context.draw_image_with_html_video_element(&video, 0.0, 0.0) {
                tracing::error!("Draw Error: {:?}", e);
                set_error_msg.set(Some("Failed to capture image".into()));
                set_is_scanning.set(false);
                return;
            }

            let data_url = html_canvas.to_data_url_with_type("image/jpeg").unwrap();
            let base64_image = data_url.split(',').nth(1).unwrap_or("").to_string();

            let existing_names: Vec<String> = existing_orchids.with_value(|orchids| {
                orchids.iter().map(|o| o.species.clone()).collect()
            });

            let zone_names: Vec<String> = zones.with_value(|z| {
                z.iter().map(|zone| zone.name.clone()).collect()
            });

            let summary = climate_readings.with_value(|readings| {
                if readings.is_empty() {
                    "No live climate data available".to_string()
                } else {
                    readings.iter().map(|r| {
                        let vpd_str = r.vpd.map(|v| format!(", {:.2} kPa VPD", v)).unwrap_or_default();
                        format!("{}: {:.1}C, {:.1}% Humidity{}", r.zone_name, r.temperature, r.humidity, vpd_str)
                    }).collect::<Vec<_>>().join(" | ")
                }
            });

            leptos::task::spawn_local(async move {
                match crate::server_fns::scanner::analyze_orchid_image(
                    base64_image,
                    Some(existing_names),
                    summary,
                    Some(zone_names),
                ).await {
                    Ok(result) => set_analysis_result.set(Some(result)),
                    Err(e) => set_error_msg.set(Some(format!("Analysis failed: {}", e))),
                }
                set_is_scanning.set(false);
            });
        }
    };

    view! {
        <div>
            {move || error_msg.get().map(|err| {
                view! { <div class="p-3 mb-4 text-sm text-red-300 rounded-lg bg-danger/20">{err}</div> }
            })}

            <div class="overflow-hidden relative mb-4 w-full bg-black rounded-xl scanner-viewfinder h-[300px]">
                <video
                    node_ref=video_element
                    autoplay
                    playsinline
                    muted
                    class="object-cover w-full h-full"
                ></video>
                <canvas node_ref=canvas_element class="hidden"></canvas>
            </div>

            <div class="scanner-controls-rise">
            {move || {
                if let Some(result) = analysis_result.get() {
                    view! { <ScanResult result=result on_add=on_add_to_collection on_reset=move || {
                        set_analysis_result.set(None);
                        set_error_msg.set(None);
                    } /> }.into_any()
                } else {
                    view! {
                        <div class="flex gap-3 justify-center mt-4 text-center">
                            <button class=BTN_GHOST on:click=flip_camera>"Flip"</button>
                            {move || {
                                if is_scanning.get() {
                                    view! {
                                        <button class="flex gap-2 items-center py-3 px-6 text-sm font-semibold text-white rounded-lg border-none cursor-not-allowed bg-primary/70" disabled>
                                            <div class="w-4 h-4 rounded-full border-2 border-white animate-spin border-t-transparent"></div>
                                            "Identifying..."
                                        </button>
                                    }.into_any()
                                } else {
                                    view! { <button class=BTN_PRIMARY on:click=capture_and_analyze>"Snap"</button> }.into_any()
                                }
                            }}
                        </div>
                    }.into_any()
                }
            }}
            </div>
        </div>
    }.into_any()
}

/// Result card with add/retry actions.
#[component]
fn ScanResult(
    result: AnalysisResult,
    on_add: impl Fn(AnalysisResult) + 'static + Copy + Send + Sync,
    on_reset: impl Fn() + 'static + Copy + Send + Sync,
    #[prop(default = "Scan Another")]
    reset_label: &'static str,
) -> impl IntoView {
    let fit_class = match result.fit_category {
        FitCategory::GoodFit => "py-1 px-3 text-sm font-semibold rounded-full bg-primary-light/20 text-primary-light",
        FitCategory::BadFit => "py-1 px-3 text-sm font-semibold rounded-full bg-danger/20 text-red-300",
        FitCategory::CautionFit => "py-1 px-3 text-sm font-semibold rounded-full bg-warning/20 text-amber-300",
    };
    let result_clone = result.clone();

    view! {
        <div class="p-5 rounded-xl bg-stone-800">
            <h3 class="mt-0 text-white">{result.species_name}</h3>
            <div class=fit_class>{result.fit_category.to_string()}</div>
            <p class="mt-3 text-sm leading-relaxed text-stone-300">{result.reason}</p>
            {result.already_owned.then(|| {
                view! { <p class="mt-2 text-sm font-semibold text-amber-400">"You already own this species!"</p> }
            })}
            <div class="grid grid-cols-2 gap-4 mt-4">
                <button class=BTN_PRIMARY on:click=move |_| on_add(result_clone.clone())>
                    "Add to Collection"
                </button>
                <button class="py-3 text-sm font-medium rounded-lg border-none transition-colors cursor-pointer text-stone-300 bg-stone-700 hover:bg-stone-600" on:click=move |_| on_reset()>
                    {reset_label}
                </button>
            </div>
        </div>
    }.into_any()
}

/// Name-based AI lookup tab — type a species name to evaluate zone fit.
#[component]
fn SearchTab(
    on_add_to_collection: impl Fn(AnalysisResult) + 'static + Copy + Send + Sync,
    existing_orchids: Vec<Orchid>,
    climate_readings: Vec<ClimateReading>,
    zones: Vec<GrowingZone>,
) -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (is_searching, set_is_searching) = signal(false);
    let (analysis_result, set_analysis_result) = signal::<Option<AnalysisResult>>(None);
    let (error_msg, set_error_msg) = signal::<Option<String>>(None);

    #[cfg(not(feature = "hydrate"))]
    {
        drop(existing_orchids);
        drop(climate_readings);
        drop(zones);
    }
    #[cfg(feature = "hydrate")]
    let existing_orchids = StoredValue::new(existing_orchids);
    #[cfg(feature = "hydrate")]
    let climate_readings = StoredValue::new(climate_readings);
    #[cfg(feature = "hydrate")]
    let zones = StoredValue::new(zones);

    let do_search = move || {
        let name = query.get().trim().to_string();
        if name.is_empty() {
            return;
        }

        set_is_searching.set(true);
        set_error_msg.set(None);
        set_analysis_result.set(None);

        #[cfg(feature = "hydrate")]
        {
            let existing_names: Vec<String> = existing_orchids.with_value(|orchids| {
                orchids.iter().map(|o| o.species.clone()).collect()
            });

            let zone_names: Vec<String> = zones.with_value(|z| {
                z.iter().map(|zone| zone.name.clone()).collect()
            });

            let summary = climate_readings.with_value(|readings| {
                if readings.is_empty() {
                    "No live climate data available".to_string()
                } else {
                    readings.iter().map(|r| {
                        let vpd_str = r.vpd.map(|v| format!(", {:.2} kPa VPD", v)).unwrap_or_default();
                        format!("{}: {:.1}C, {:.1}% Humidity{}", r.zone_name, r.temperature, r.humidity, vpd_str)
                    }).collect::<Vec<_>>().join(" | ")
                }
            });

            leptos::task::spawn_local(async move {
                match crate::server_fns::scanner::analyze_orchid_by_name(
                    name,
                    Some(existing_names),
                    summary,
                    Some(zone_names),
                ).await {
                    Ok(result) => set_analysis_result.set(Some(result)),
                    Err(e) => set_error_msg.set(Some(format!("Lookup failed: {}", e))),
                }
                set_is_searching.set(false);
            });
        }
    };

    let on_submit = move |_| {
        do_search();
    };

    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Enter" {
            do_search();
        }
    };

    view! {
        <div>
            {move || error_msg.get().map(|err| {
                view! { <div class="p-3 mb-4 text-sm text-red-300 rounded-lg bg-danger/20">{err}</div> }
            })}

            {move || if let Some(result) = analysis_result.get() {
                view! {
                    <ScanResult result=result on_add=on_add_to_collection on_reset=move || {
                        set_analysis_result.set(None);
                        set_error_msg.set(None);
                        set_query.set(String::new());
                    } reset_label="Search Another" />
                }.into_any()
            } else {
                view! {
                    <div>
                        <div class="flex gap-2 mb-4">
                            <div class="relative flex-1">
                                <input
                                    type="text"
                                    class=SEARCH_INPUT
                                    placeholder="e.g. Phalaenopsis bellina"
                                    prop:value=query
                                    on:input=move |ev| set_query.set(event_target_value(&ev))
                                    on:keydown=on_keydown
                                    disabled=move || is_searching.get()
                                />
                                <div class="flex absolute top-0 right-0 justify-center items-center px-3 h-full pointer-events-none text-stone-500">
                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                        <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
                                    </svg>
                                </div>
                            </div>
                            {move || if is_searching.get() {
                                view! {
                                    <button class="flex gap-2 items-center py-3 px-5 text-sm font-semibold text-white rounded-xl border-none cursor-not-allowed bg-primary/70 shrink-0" disabled>
                                        <div class="w-4 h-4 rounded-full border-2 border-white animate-spin border-t-transparent"></div>
                                        "Looking up..."
                                    </button>
                                }.into_any()
                            } else {
                                view! {
                                    <button class=format!("{} shrink-0", BTN_PRIMARY)
                                        disabled=move || query.get().trim().is_empty()
                                        on:click=on_submit
                                    >"Look Up"</button>
                                }.into_any()
                            }}
                        </div>

                        <div class="py-8 text-center">
                            <div class="mb-3 text-3xl opacity-20">{"\u{1F3F7}\u{FE0F}"}</div>
                            <p class="text-sm text-stone-500">"Type a species name and we'll check if it suits your zones"</p>
                        </div>
                    </div>
                }.into_any()
            }}
        </div>
    }.into_any()
}
