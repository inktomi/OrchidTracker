use leptos::prelude::*;
use leptos::task::spawn_local;
use gloo_storage::{LocalStorage, Storage};
use web_sys::{HtmlCanvasElement, MediaStreamConstraints};
use wasm_bindgen::JsCast;
use serde::{Deserialize, Serialize};
use crate::orchid::{Orchid, FitCategory, LightRequirement};
use crate::app::ClimateData;
use wasm_bindgen_futures::JsFuture;

const MODAL_OVERLAY: &str = "fixed inset-0 bg-black/50 flex justify-center items-center z-[1000]";
const MODAL_HEADER: &str = "flex justify-between items-center mb-4 border-b border-gray-600 pb-2";
const CLOSE_BTN: &str = "bg-gray-400 text-white border-none py-2 px-3 rounded cursor-pointer hover:bg-gray-500";

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
}

fn extract_gemini_text(json: &serde_json::Value) -> Option<String> {
    json.get("candidates")?
        .get(0)?
        .get("content")?
        .get("parts")?
        .get(0)?
        .get("text")?
        .as_str()
        .map(|s| s.replace("```json", "").replace("```", "").trim().to_string())
}

#[component]
pub fn ScannerModal(
    on_close: impl Fn() + 'static + Copy + Send + Sync,
    on_add_to_collection: impl Fn(AnalysisResult) + 'static + Copy + Send + Sync,
    existing_orchids: Vec<Orchid>,
    climate_data: Vec<ClimateData>,
) -> impl IntoView {
    let (is_scanning, set_is_scanning) = signal(false);
    let (analysis_result, set_analysis_result) = signal::<Option<AnalysisResult>>(None);
    let (error_msg, set_error_msg) = signal::<Option<String>>(None);

    let video_element: NodeRef<leptos::html::Video> = NodeRef::new();
    let canvas_element: NodeRef<leptos::html::Canvas> = NodeRef::new();

    let existing_orchids = StoredValue::new(existing_orchids);
    let climate_data = StoredValue::new(climate_data);

    let (facing_mode, set_facing_mode) = signal("environment".to_string());
    let (stream_signal, set_stream_signal) = signal_local::<Option<web_sys::MediaStream>>(None);

    // Stop camera cleanup
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

    // Start/Restart Camera when facing_mode changes
    Effect::new(move |_| {
        let mode = facing_mode.get();

        // Stop previous stream
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

             spawn_local(async move {
                if let Ok(media_devices) = navigator.media_devices() {
                    let constraints = MediaStreamConstraints::new();

                    let video_constraint = js_sys::Object::new();
                    let _ = js_sys::Reflect::set(&video_constraint, &"facingMode".into(), &mode.into());

                    constraints.set_video(&video_constraint);

                    match media_devices.get_user_media_with_constraints(&constraints) {
                        Ok(promise) => {
                            if let Ok(stream_js) = JsFuture::from(promise).await {
                                let stream = stream_js.unchecked_into::<web_sys::MediaStream>();
                                video.set_src_object(Some(&stream));
                                let _ = video.play();
                                set_stream_signal.set(Some(stream));
                            }
                        }
                        Err(e) => {
                            log::error!("Camera Error: {:?}", e);
                            set_error_msg.set(Some("Camera access denied or not available.".into()));
                        }
                    }
                }
             });
        }
    });

    let flip_camera = move |_| {
        set_facing_mode.update(|m| *m = if m == "environment" { "user".into() } else { "environment".into() });
    };

    let capture_and_analyze = move |_| {
        set_is_scanning.set(true);
        set_error_msg.set(None);
        set_analysis_result.set(None);

        spawn_local(async move {
            let raw_key = LocalStorage::get("gemini_api_key").unwrap_or_else(|_| String::new());
            let api_key = raw_key.trim();
            
            if api_key.is_empty() {
                set_error_msg.set(Some("Gemini API Key missing in Settings".into()));
                set_is_scanning.set(false);
                return;
            }

            let video = video_element.get().expect("Video element missing");
            let canvas = canvas_element.get().expect("Canvas element missing");
            let html_canvas: &HtmlCanvasElement = &canvas;

            let context = html_canvas.get_context("2d").unwrap().unwrap().unchecked_into::<web_sys::CanvasRenderingContext2d>();

            let width = video.video_width() as f64;
            let height = video.video_height() as f64;
            html_canvas.set_width(width as u32);
            html_canvas.set_height(height as u32);

            if let Err(e) = context.draw_image_with_html_video_element(&video, 0.0, 0.0) {
                 log::error!("Draw Error: {:?}", e);
                 set_error_msg.set(Some("Failed to capture image".into()));
                 set_is_scanning.set(false);
                 return;
            }

            let data_url = html_canvas.to_data_url().unwrap();
            let base64_image = data_url.split(',').nth(1).unwrap_or("").to_string();

             let client = reqwest::Client::new();
             let existing_names: Vec<String> = existing_orchids.with_value(|orchids| {
                 orchids.iter().map(|o| o.species.clone()).collect()
             });
             
             let summary = climate_data.with_value(|cd| {
                 if cd.is_empty() {
                     "Unknown climate".to_string()
                 } else {
                     cd.iter().map(|d| {
                         format!("{}: {}C, {}% Humid, {}kPa VPD", d.name, d.temperature, d.humidity, d.vpd)
                     }).collect::<Vec<_>>().join(" | ")
                 }
             });

             let prompt = format!(
                 "Identify the orchid species from this image. \
                 Think step-by-step: \
                 1. Identify the species with high confidence (look for tags). \
                 2. Analyze its natural habitat and care requirements. \
                 3. Compare requirements against my conditions: {}. \
                 4. Consider outdoor conditions (90606, partial shade patio or full sun outdoor rack). Outdoor Rack: High Sun with shade cloth. Patio: Morning Sun/Afternoon Shade. \
                 5. Check if I own it already. I own these plants: {:?}. \
                 Then, evaluate the fit of this new plant with my existing conditions. \
                 Finally, return ONLY valid JSON with this structure (no markdown): \
                 {{ \"species_name\": \"...\", \"fit_category\": \"Good Fit\", \"reason\": \"...\", \"already_owned\": false, \"water_freq\": 7, \"light_req\": \"Medium\", \"temp_range\": \"18-28C\", \"placement_suggestion\": \"Medium\", \"conservation_status\": \"CITES II\" }} \
                 Allowed fit_categories: 'Good Fit', 'Bad Fit', 'Caution Fit'. \
                 For light_req, choose from: 'High', 'Medium', 'Low'. \
                 For placement_suggestion, choose from: 'High', 'Medium', 'Low', 'Patio', 'OutdoorRack'. \
                 For conservation_status, use 'CITES I', 'CITES II', 'Endangered', 'Vulnerable', or null if unknown/common.",
                 summary,
                 existing_names
             );

             let request_body = serde_json::json!({
                 "contents": [{
                     "parts": [
                         { "text": prompt },
                         { "inline_data": { "mime_type": "image/jpeg", "data": base64_image } }
                     ]
                 }]
             });

             let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-3.0-flash:generateContent?key={}", api_key);
             
             // Debug log (mask key)
             log::info!("Sending AI request to: {}...", url.split("?key=").next().unwrap_or("..."));

             match client.post(&url)
                .json(&request_body)
                .send()
                .await {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            match resp.json::<serde_json::Value>().await {
                                Ok(json_resp) => {
                                    if let Some(clean_text) = extract_gemini_text(&json_resp) {
                                        match serde_json::from_str::<AnalysisResult>(&clean_text) {
                                            Ok(result) => set_analysis_result.set(Some(result)),
                                            Err(e) => {
                                                set_error_msg.set(Some(format!("Failed to parse AI response: {}", e)));
                                                log::error!("Raw AI text: {}", clean_text);
                                            }
                                        }
                                    } else {
                                        set_error_msg.set(Some("Could not extract text from AI response".into()));
                                    }
                                }
                                Err(_) => set_error_msg.set(Some("Invalid response format from AI".into())),
                            }
                        } else {
                             let status = resp.status();
                             let error_body = resp.text().await.unwrap_or_else(|_| "(no content)".into());
                             log::error!("AI API Error {} Body: {}", status, error_body);
                             
                             let msg = format!("AI API Error: {} - Details: {}", status, error_body);
                             set_error_msg.set(Some(msg));
                        }
                    },
                    Err(e) => set_error_msg.set(Some(format!("Network Error: {}", e))),
                }

            set_is_scanning.set(false);
        });
    };

    view! {
        <div class=MODAL_OVERLAY>
            <div class="bg-neutral-900 text-gray-200 p-8 rounded-lg w-[90%] max-w-[600px] max-h-[90vh] overflow-y-auto border border-neutral-700">
                 <div class=MODAL_HEADER>
                    <h2 class="m-0 text-white">"Orchid Scanner"</h2>
                    <button class=CLOSE_BTN on:click=move |_| on_close()>"X"</button>
                </div>
                <div>
                    {move || error_msg.get().map(|err| {
                        view! { <div class="mb-2 text-red-400">{err}</div> }
                    })}

                    <div class="overflow-hidden relative mb-4 w-full bg-black rounded-lg h-[300px]">
                        <video
                            node_ref=video_element
                            autoplay
                            playsinline
                            muted
                            class="object-cover w-full h-full"
                        ></video>
                        <canvas node_ref=canvas_element class="hidden"></canvas>
                    </div>

                    <div>
                    {move || {
                        if let Some(result) = analysis_result.get() {
                            let fit_class = match result.fit_category {
                                FitCategory::GoodFit => "py-1 px-3 rounded-xl inline-block mb-2 text-sm bg-green-100 text-primary",
                                FitCategory::BadFit => "py-1 px-3 rounded-xl inline-block mb-2 text-sm bg-red-100 text-danger",
                                FitCategory::CautionFit => "py-1 px-3 rounded-xl inline-block mb-2 text-sm bg-orange-100 text-warning",
                            };
                            let result_clone = result.clone();

                            view! {
                                <div class="p-4 rounded-lg bg-neutral-800">
                                    <h3>{result.species_name}</h3>
                                    <div class=fit_class>{result.fit_category.to_string()}</div>
                                    <p>{result.reason}</p>
                                    {result.already_owned.then(|| {
                                        view! { <p class="font-bold text-yellow-400">"You already own this species!"</p> }
                                    })}
                                    <div class="grid grid-cols-2 gap-4 mt-4">
                                        <button class="py-3 text-white rounded border-none cursor-pointer bg-primary hover:bg-primary-dark" on:click=move |_| on_add_to_collection(result_clone.clone())>
                                            "Use Info"
                                        </button>
                                        <button class="py-3 text-white rounded border-none cursor-pointer bg-neutral-600 hover:bg-neutral-500" on:click=move |_| {
                                            set_analysis_result.set(None);
                                            set_error_msg.set(None);
                                        }>"Scan Another"</button>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="flex gap-4 justify-center mt-4 text-center">
                                    <button class="py-2 px-3 font-bold text-white bg-transparent rounded border cursor-pointer border-white/80 hover:bg-white/20" on:click=flip_camera>"Flip"</button>
                                    {move || {
                                        if is_scanning.get() {
                                            view! { <button class="py-3 px-6 text-white rounded border-none cursor-pointer bg-primary" disabled>"Analyzing..."</button> }.into_any()
                                        } else {
                                            view! { <button class="py-3 px-6 text-white rounded border-none cursor-pointer bg-primary hover:bg-primary-dark" on:click=capture_and_analyze>"Capture"</button> }.into_any()
                                        }
                                    }}
                                </div>
                            }.into_any()
                        }
                    }}
                    </div>
                </div>
            </div>
        </div>
    }
}
