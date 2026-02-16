use leptos::*;
use gloo_storage::{LocalStorage, Storage};
use web_sys::{HtmlVideoElement, HtmlCanvasElement, MediaStreamConstraints};
use wasm_bindgen::JsCast;
use serde::{Deserialize, Serialize};
use crate::orchid::Orchid;
use gloo_timers::future::spawn_local;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnalysisResult {
    pub species_name: String,
    pub fit_category: String, // "Good Fit", "Bad Fit", "Caution Fit"
    pub reason: String,
    pub already_owned: bool,
    // Add fields to pre-fill the form
    pub water_freq: u32,
    pub light_req: String,
    pub temp_range: String,
    pub placement_suggestion: String,
}

#[component]
pub fn ScannerModal<F, A>(
    on_close: F,
    on_add_to_collection: A,
    existing_orchids: Vec<Orchid>,
    climate_summary: String
) -> impl IntoView
where
    F: Fn() + 'static + Copy,
    A: Fn(AnalysisResult) + 'static + Copy,
{
    let (is_scanning, set_is_scanning) = create_signal(false);
    let (analysis_result, set_analysis_result) = create_signal::<Option<AnalysisResult>>(None);
    let (error_msg, set_error_msg) = create_signal::<Option<String>>(None);
    
    // Node references for video and canvas
    let video_element: NodeRef<html::Video> = create_node_ref();
    let canvas_element: NodeRef<html::Canvas> = create_node_ref();

    // Start Camera on Mount
    create_effect(move |_| {
        if let Some(video) = video_element.get() {
             let window = web_sys::window().unwrap();
             let navigator = window.navigator();
             
             spawn_local(async move {
                if let Ok(media_devices) = navigator.media_devices() {
                    let mut constraints = MediaStreamConstraints::new();
                    // Prefer environment facing camera
                    let constraint_obj = js_sys::Object::new();
                    let video_constraint = js_sys::Object::new();
                    let _ = js_sys::Reflect::set(&video_constraint, &"facingMode".into(), &"environment".into());
                    let _ = js_sys::Reflect::set(&constraint_obj, &"video".into(), &video_constraint);

                    // Use constraints directly if possible, but web-sys types are tricky. 
                    // Let's use basic constraints first to ensure it compiles, then refine.
                    constraints.video(&wasm_bindgen::JsValue::TRUE);

                    match media_devices.get_user_media_with_constraints(&constraints) {
                        Ok(promise) => {
                            if let Ok(stream_js) = wasm_bindgen_futures::JsFuture::from(promise).await {
                                let stream = stream_js.unchecked_into::<web_sys::MediaStream>();
                                video.set_src_object(Some(&stream));
                                let _ = video.play();
                            }
                        }
                        Err(e) => {
                            log::error!("Camera Error: {:?}", e);
                            set_error_msg.set(Some("Camera access denied or not available.".to_string()));
                        }
                    }
                }
             });
        }
    });

    let capture_and_analyze = move |_| {
        set_is_scanning.set(true);
        set_error_msg.set(None);
        set_analysis_result.set(None);

        spawn_local(async move {
            let api_key = LocalStorage::get("gemini_api_key").unwrap_or_else(|_| "".to_string());
            if api_key.is_empty() {
                set_error_msg.set(Some("Gemini API Key missing in Settings".to_string()));
                set_is_scanning.set(false);
                return;
            }

            // Capture Frame
            let video = video_element.get().expect("Video element missing");
            let canvas = canvas_element.get().expect("Canvas element missing");
            // Cast canvas to HtmlCanvasElement to get context
            let html_canvas: HtmlCanvasElement = canvas.clone().into_any().unchecked_into();
            
            let context = html_canvas.get_context("2d").unwrap().unwrap().unchecked_into::<web_sys::CanvasRenderingContext2d>();
            
            let width = video.video_width() as f64;
            let height = video.video_height() as f64;
            html_canvas.set_width(width as u32);
            html_canvas.set_height(height as u32);
            
            if let Err(e) = context.draw_image_with_html_video_element(&video, 0.0, 0.0) {
                 log::error!("Draw Error: {:?}", e);
                 set_error_msg.set(Some("Failed to capture image".to_string()));
                 set_is_scanning.set(false);
                 return;
            }
            
            let data_url = html_canvas.to_data_url().unwrap();
            let base64_image = data_url.split(",").nth(1).unwrap_or("").to_string();

            // Prepare AI Request
             let client = reqwest::Client::new();
             let existing_names: Vec<String> = existing_orchids.iter().map(|o| o.species.clone()).collect();
             let prompt = format!(
                 "Identify the orchid species from this image. \
                 Then, evaluate if it is a good fit for my conditions: {}. \
                 Also check if I already own it (My List: {:?}). \
                 Return ONLY valid JSON with this structure (no markdown): \
                 {{ \"species_name\": \"...\", \"fit_category\": \"Good Fit\", \"reason\": \"...\", \"already_owned\": false, \"water_freq\": 7, \"light_req\": \"Medium\", \"temp_range\": \"18-28C\", \"placement_suggestion\": \"Medium\" }} \
                 Allowed fit_categories: 'Good Fit', 'Bad Fit', 'Caution Fit'.",
                 climate_summary,
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
                            if let Ok(json_resp) = resp.json::<serde_json::Value>().await {
                                // Parse Gemini Response safely
                                if let Some(candidates) = json_resp.get("candidates") {
                                    if let Some(first) = candidates.get(0) {
                                        if let Some(content) = first.get("content") {
                                             if let Some(parts) = content.get("parts") {
                                                 if let Some(text_part) = parts.get(0) {
                                                     if let Some(text) = text_part.get("text") {
                                                         let clean_text = text.as_str().unwrap_or("").replace("```json", "").replace("```", "").trim().to_string();
                                                         match serde_json::from_str::<AnalysisResult>(&clean_text) {
                                                             Ok(result) => set_analysis_result.set(Some(result)),
                                                             Err(e) => {
                                                                 set_error_msg.set(Some(format!("Failed to parse AI response: {}", e)));
                                                                 log::error!("Raw AI text: {}", clean_text);
                                                             }
                                                         }
                                                     }
                                                 }
                                             }
                                        }
                                    }
                                }
                            } else {
                                set_error_msg.set(Some("Invalid response format from AI".to_string()));
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
                    {move || if let Some(err) = error_msg.get() {
                        view! { <div class="error-msg">{err}</div> }.into_view()
                    } else {
                        view! {}.into_view()
                    }}
                    
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
                    {move || if let Some(result) = analysis_result.get() {
                        let fit_class = match result.fit_category.as_str() {
                            "Good Fit" => "status-ok fit-badge",
                            "Bad Fit" => "status-error fit-badge",
                             _ => "status-warning fit-badge"
                        };
                        
                        view! {
                            <div class="scan-result-card">
                                <h3>{result.species_name}</h3>
                                <div class=fit_class>{result.fit_category}</div>
                                <p class="reason-text">{result.reason}</p>
                                {if result.already_owned {
                                    view! { <p class="owned-warning">"⚠️ You already own this species!"</p> }.into_view()
                                } else {
                                    view! {}.into_view()
                                }}
                                <div class="action-buttons">
                                    <button class="action-btn" on:click=move |_| on_add_to_collection(result.clone())>
                                        "Use Info"
                                    </button>
                                    <button class="retry-btn" on:click=move |_| {
                                        set_analysis_result.set(None);
                                        set_error_msg.set(None);
                                    }>"Scan Another"</button>
                                </div>
                            </div>
                        }.into_view()
                    } else {
                        view! {
                            <div class="controls" style="margin-top: 1rem; text-align: center;">
                                {if is_scanning.get() {
                                    view! { <button disabled>"Analyzing..."</button> }.into_view()
                                } else {
                                    view! { <button on:click=capture_and_analyze>"Capture & Analyze"</button> }.into_view()
                                }}
                            </div>
                        }.into_view()
                    }}
                    </div>
                </div>
            </div>
        </div>
    }
}
