use leptos::prelude::*;
use leptos::task::spawn_local;
use gloo_storage::{LocalStorage, Storage};
use web_sys::{HtmlCanvasElement, MediaStreamConstraints};
use wasm_bindgen::JsCast;
use serde::{Deserialize, Serialize};
use crate::orchid::{Orchid, FitCategory, LightRequirement};
use wasm_bindgen_futures::JsFuture;

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    climate_summary: String,
) -> impl IntoView {
    let (is_scanning, set_is_scanning) = signal(false);
    let (analysis_result, set_analysis_result) = signal::<Option<AnalysisResult>>(None);
    let (error_msg, set_error_msg) = signal::<Option<String>>(None);

    let video_element: NodeRef<leptos::html::Video> = NodeRef::new();
    let canvas_element: NodeRef<leptos::html::Canvas> = NodeRef::new();

    let existing_orchids = StoredValue::new(existing_orchids);
    let climate_summary = StoredValue::new(climate_summary);

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
            let api_key = LocalStorage::get("gemini_api_key").unwrap_or_else(|_| String::new());
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
             let summary = climate_summary.get_value();

             let prompt = format!(
                 "Identify the orchid species from this image. \
                 Then, evaluate if it is a good fit for my conditions: {}. \
                 I also have outdoor space in zip code 90606 (Outdoor Rack: High Sun but with some shade cloth protection, still very exposed - Laelia anceps does well here; Patio: Morning Sun/Afternoon Shade). \
                 Also check if I already own it (My List: {:?}). \
                 Return ONLY valid JSON with this structure (no markdown): \
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

             let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}", api_key);

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
                             set_error_msg.set(Some(format!("AI API Error: {}", resp.status())));
                        }
                    },
                    Err(e) => set_error_msg.set(Some(format!("Network Error: {}", e))),
                }

            set_is_scanning.set(false);
        });
    };

    view! {
        <div class="modal-overlay">
            <div class="modal-content scanner-modal">
                 <div class="modal-header">
                    <h2>"Orchid Scanner"</h2>
                    <button class="close-btn" on:click=move |_| on_close()>"X"</button>
                </div>
                <div class="modal-body">
                    {move || error_msg.get().map(|err| {
                        view! { <div class="error-msg">{err}</div> }
                    })}

                    <div class="camera-container" style="position: relative; width: 100%; height: 300px; background: #000;">
                        <video
                            node_ref=video_element
                            autoplay
                            playsinline
                            muted
                            style="width: 100%; height: 100%; object-fit: cover;"
                        ></video>
                        <canvas node_ref=canvas_element style="display:none;"></canvas>
                    </div>

                    <div class="scan-results-area">
                    {move || {
                        if let Some(result) = analysis_result.get() {
                            let fit_class = match result.fit_category {
                                FitCategory::GoodFit => "status-ok fit-badge",
                                FitCategory::BadFit => "status-error fit-badge",
                                FitCategory::CautionFit => "status-warning fit-badge",
                            };
                            let result_clone = result.clone();

                            view! {
                                <div class="scan-result-card">
                                    <h3>{result.species_name}</h3>
                                    <div class=fit_class>{result.fit_category.to_string()}</div>
                                    <p class="reason-text">{result.reason}</p>
                                    {result.already_owned.then(|| {
                                        view! { <p class="owned-warning">"You already own this species!"</p> }
                                    })}
                                    <div class="action-buttons">
                                        <button class="action-btn" on:click=move |_| on_add_to_collection(result_clone.clone())>
                                            "Use Info"
                                        </button>
                                        <button class="retry-btn" on:click=move |_| {
                                            set_analysis_result.set(None);
                                            set_error_msg.set(None);
                                        }>"Scan Another"</button>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="controls" style="margin-top: 1rem; text-align: center; display: flex; gap: 1rem; justify-content: center;">
                                    <button class="action-btn" on:click=flip_camera>"Flip"</button>
                                    {move || {
                                        if is_scanning.get() {
                                            view! { <button disabled>"Analyzing..."</button> }.into_any()
                                        } else {
                                            view! { <button on:click=capture_and_analyze>"Capture"</button> }.into_any()
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
