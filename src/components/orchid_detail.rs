use leptos::prelude::*;
use chrono::Datelike;
use crate::orchid::{Orchid, LightRequirement, GrowingZone, ClimateReading, LogEntry, Hemisphere, SeasonalPhase, month_in_range};
use crate::watering::ClimateSnapshot;
use crate::components::habitat_weather::HabitatWeatherCard;
use crate::components::quick_actions::QuickActions;
use crate::components::photo_capture::PhotoCapture;
use crate::components::growth_thread::GrowthThread;
use crate::components::first_bloom::FirstBloomCelebration;
use crate::components::photo_gallery::PhotoGallery;
use super::{MODAL_OVERLAY, MODAL_CONTENT, MODAL_HEADER, BTN_PRIMARY, BTN_SECONDARY, BTN_CLOSE};

/// Serialize an enum to its serde variant name (e.g., PotType::Mounted → "Mounted").
/// Used to populate edit form dropdowns whose option values match serde names.
fn serde_variant_name<T: serde::Serialize>(val: &T) -> String {
    serde_json::to_string(val)
        .unwrap_or_default()
        .trim_matches('"')
        .to_string()
}

const EDIT_BTN: &str = "py-2 px-3 text-sm font-semibold text-white rounded-lg border-none cursor-pointer bg-accent hover:bg-accent-dark transition-colors";
const TAB_ACTIVE: &str = "py-2 px-4 text-sm font-semibold border-b-2 cursor-pointer transition-colors text-primary border-primary bg-transparent";
const TAB_INACTIVE: &str = "py-2 px-4 text-sm font-medium border-b-2 border-transparent cursor-pointer transition-colors text-stone-400 hover:text-stone-600 bg-transparent dark:hover:text-stone-300";

fn light_req_to_key(lr: &LightRequirement) -> String {
    lr.as_str().to_string()
}

#[derive(Clone, Copy, PartialEq)]
enum DetailTab {
    Journal,
    Gallery,
    Details,
}

#[component]
pub fn OrchidDetail(
    orchid: Orchid,
    zones: Vec<GrowingZone>,
    climate_readings: Vec<ClimateReading>,
    #[prop(default = Vec::new())] climate_snapshots: Vec<ClimateSnapshot>,
    hemisphere: String,
    on_close: impl Fn() + 'static + Send + Sync,
    on_update: impl Fn(Orchid) + 'static + Copy + Send + Sync,
    #[prop(optional)] read_only: bool,
    #[prop(optional)] public_username: Option<String>,
) -> impl IntoView {
    let (orchid_signal, set_orchid_signal) = signal(orchid.clone());
    let (log_entries, set_log_entries) = signal(Vec::<LogEntry>::new());
    let (active_tab, set_active_tab) = signal(DetailTab::Journal);
    let (show_first_bloom, set_show_first_bloom) = signal(false);

    // Load log entries on mount
    {
        let orchid_id = orchid.id.clone();
        let pub_user = public_username;
        leptos::task::spawn_local(async move {
            let result = if let Some(uname) = pub_user {
                crate::server_fns::public::get_public_log_entries(uname, orchid_id).await
            } else {
                crate::server_fns::orchids::get_log_entries(orchid_id).await
            };
            match result {
                Ok(entries) => set_log_entries.set(entries),
                Err(e) => tracing::error!("Failed to load log entries: {}", e),
            }
        });
    }

    // Edit mode state
    let (is_editing, set_is_editing) = signal(false);
    let zones_stored = StoredValue::new(zones);
    let hemisphere_stored = StoredValue::new(hemisphere);

    // Climate snapshot for this orchid's zone
    let climate_snapshot_stored = StoredValue::new({
        let placement = orchid.placement.clone();
        climate_snapshots.into_iter().find(|s| s.zone_name == placement)
    });

    // Habitat weather data
    let habitat_zone_reading = StoredValue::new({
        let placement = orchid.placement.clone();
        climate_readings.into_iter().find(|r| r.zone_name == placement)
    });
    let native_region = StoredValue::new(orchid.native_region.clone());
    let native_lat = orchid.native_latitude;
    let native_lon = orchid.native_longitude;

    view! {
        <div class=MODAL_OVERLAY>
            <div class=MODAL_CONTENT>
                // Header
                <div class=MODAL_HEADER>
                    <div>
                        <h2 class="m-0">{move || orchid_signal.get().name}</h2>
                        <p class="mt-0.5 mb-0 text-sm italic text-stone-500 dark:text-stone-400">{move || orchid_signal.get().species}</p>
                    </div>
                    <div class="flex gap-2">
                        <button class=BTN_CLOSE aria-label="Close details" title="Close" on:click=move |_| on_close()>"\u{00D7}"</button>
                    </div>
                </div>

                // Tab bar
                <div class="flex gap-0 mb-4 border-b border-stone-200 dark:border-stone-700">
                    <button
                        class=move || if active_tab.get() == DetailTab::Journal { TAB_ACTIVE } else { TAB_INACTIVE }
                        on:click=move |_| set_active_tab.set(DetailTab::Journal)
                    >"Journal"</button>
                    <button
                        class=move || if active_tab.get() == DetailTab::Gallery { TAB_ACTIVE } else { TAB_INACTIVE }
                        on:click=move |_| set_active_tab.set(DetailTab::Gallery)
                    >"Gallery"</button>
                    <button
                        class=move || if active_tab.get() == DetailTab::Details { TAB_ACTIVE } else { TAB_INACTIVE }
                        on:click=move |_| set_active_tab.set(DetailTab::Details)
                    >"Details"</button>
                </div>

                // Tab content
                <div>
                    {move || match active_tab.get() {
                        DetailTab::Journal => view! {
                            <JournalTab
                                orchid_signal=orchid_signal
                                set_orchid_signal=set_orchid_signal
                                log_entries=log_entries
                                set_log_entries=set_log_entries
                                set_show_first_bloom=set_show_first_bloom
                                read_only=read_only
                            />
                        }.into_any(),
                        DetailTab::Gallery => view! {
                            <PhotoGallery entries=log_entries />
                        }.into_any(),
                        DetailTab::Details => view! {
                            <DetailsTab
                                orchid_signal=orchid_signal
                                set_orchid_signal=set_orchid_signal
                                is_editing=is_editing
                                set_is_editing=set_is_editing
                                zones=zones_stored
                                hemisphere=hemisphere_stored
                                climate_snapshot=climate_snapshot_stored
                                on_update=on_update
                                set_log_entries=set_log_entries
                                habitat_zone_reading=habitat_zone_reading
                                native_region=native_region
                                native_lat=native_lat
                                native_lon=native_lon
                                read_only=read_only
                            />
                        }.into_any(),
                    }}
                </div>
            </div>
        </div>

        // First bloom celebration overlay
        {move || show_first_bloom.get().then(|| {
            view! {
                <FirstBloomCelebration on_dismiss=move || set_show_first_bloom.set(false) />
            }
        })}
    }
}

// ── Journal Tab ──────────────────────────────────────────────────────

#[component]
fn JournalTab(
    orchid_signal: ReadSignal<Orchid>,
    set_orchid_signal: WriteSignal<Orchid>,
    log_entries: ReadSignal<Vec<LogEntry>>,
    set_log_entries: WriteSignal<Vec<LogEntry>>,
    set_show_first_bloom: WriteSignal<bool>,
    #[prop(optional)] read_only: bool,
) -> impl IntoView {
    let (note, set_note) = signal(String::new());
    // Staged photo data URL — NOT uploaded until the form is submitted
    let (staged_photo, set_staged_photo) = signal(Option::<String>::None);
    let (is_syncing, set_is_syncing) = signal(false);
    // Bumped after successful save to reset PhotoCapture preview
    let (photo_reset, set_photo_reset) = signal(0u32);

    let on_submit_note = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let current_note = note.get();
        let photo_data_url = staged_photo.get();

        // Require at least a note or photo
        if current_note.is_empty() && photo_data_url.is_none() {
            return;
        }

        set_is_syncing.set(true);
        let orchid_id = orchid_signal.get().id.clone();

        leptos::task::spawn_local(async move {
            // Upload staged photo first (if any), then create the log entry
            let server_filename = if let Some(_data_url) = photo_data_url {
                #[cfg(feature = "hydrate")]
                {
                    match crate::components::photo_capture::upload_data_url(&_data_url).await {
                        Ok(fname) => Some(fname),
                        Err(e) => {
                            tracing::error!("Photo upload failed: {}", e);
                            set_is_syncing.set(false);
                            return;
                        }
                    }
                }
                #[cfg(not(feature = "hydrate"))]
                { None }
            } else {
                None
            };

            match crate::server_fns::orchids::add_log_entry(
                orchid_id,
                current_note,
                server_filename,
                None,
            ).await {
                Ok(response) => {
                    if response.is_first_bloom {
                        set_show_first_bloom.set(true);
                    }
                    set_log_entries.update(|entries| entries.insert(0, response.entry));
                }
                Err(e) => tracing::error!("Failed to add note: {}", e),
            }
            set_is_syncing.set(false);
            set_note.set(String::new());
            set_staged_photo.set(None);
            set_photo_reset.update(|v| *v += 1);
        });
    };

    let clear_staged = std::sync::Arc::new(move || {
        set_staged_photo.set(None);
    }) as std::sync::Arc<dyn Fn() + Send + Sync>;

    view! {
        // Quick Actions + Detailed Note form (hidden in read-only mode)
        {(!read_only).then(|| view! {
            <QuickActions
                orchid_signal=orchid_signal
                set_orchid_signal=set_orchid_signal
                set_log_entries=set_log_entries
                set_show_first_bloom=set_show_first_bloom
            />

            <div class="p-4 mb-6 rounded-xl border border-stone-200 dark:border-stone-700">
                <h4 class="mt-0 mb-3 text-xs font-semibold tracking-widest uppercase text-stone-500 dark:text-stone-400">"Add a detailed note"</h4>
                <form on:submit=on_submit_note>
                    // Photo capture — stages locally, upload deferred to submit
                    <div class="mb-3">
                        <PhotoCapture
                            on_photo_ready=move |data_url| set_staged_photo.set(Some(data_url))
                            on_clear=clear_staged.clone()
                            reset=photo_reset
                        />
                    </div>

                    // Note textarea
                    <div class="mb-3">
                        <textarea
                            prop:value=note
                            on:input=move |ev| set_note.set(event_target_value(&ev))
                            placeholder="Write a note about this orchid..."
                            rows="2"
                            class="py-2 px-3 w-full text-sm bg-white rounded-lg border border-stone-300 dark:bg-stone-800 dark:border-stone-600 dark:text-stone-200"
                        ></textarea>
                    </div>

                    <button type="submit" class=BTN_PRIMARY disabled=move || is_syncing.get()>
                        {move || if is_syncing.get() { "Uploading..." } else { "Add Note" }}
                    </button>
                </form>
            </div>
        })}

        // Growth Thread
        <GrowthThread entries=log_entries orchid_id=orchid_signal.get_untracked().id />
    }.into_any()
}

// ── Details Tab ──────────────────────────────────────────────────────

#[component]
fn DetailsTab(
    orchid_signal: ReadSignal<Orchid>,
    set_orchid_signal: WriteSignal<Orchid>,
    is_editing: ReadSignal<bool>,
    set_is_editing: WriteSignal<bool>,
    zones: StoredValue<Vec<GrowingZone>>,
    hemisphere: StoredValue<String>,
    climate_snapshot: StoredValue<Option<ClimateSnapshot>>,
    on_update: impl Fn(Orchid) + 'static + Copy + Send + Sync,
    set_log_entries: WriteSignal<Vec<LogEntry>>,
    habitat_zone_reading: StoredValue<Option<ClimateReading>>,
    native_region: StoredValue<Option<String>>,
    native_lat: Option<f64>,
    native_lon: Option<f64>,
    #[prop(optional)] read_only: bool,
) -> impl IntoView {
    let (is_watering, set_is_watering) = signal(false);

    // Edit form signals
    let (edit_name, set_edit_name) = signal(String::new());
    let (edit_species, set_edit_species) = signal(String::new());
    let (edit_water_freq, set_edit_water_freq) = signal(String::new());
    let (edit_light_req, set_edit_light_req) = signal(String::new());
    let (edit_placement, set_edit_placement) = signal(String::new());
    let (edit_light_lux, set_edit_light_lux) = signal(String::new());
    let (edit_temp_range, set_edit_temp_range) = signal(String::new());
    let (edit_notes, set_edit_notes) = signal(String::new());
    let (edit_conservation, set_edit_conservation) = signal(String::new());
    let (edit_temp_min, set_edit_temp_min) = signal(String::new());
    let (edit_temp_max, set_edit_temp_max) = signal(String::new());
    let (edit_humidity_min, set_edit_humidity_min) = signal(String::new());
    let (edit_humidity_max, set_edit_humidity_max) = signal(String::new());
    let (edit_fert_freq, set_edit_fert_freq) = signal(String::new());
    let (edit_fert_type, set_edit_fert_type) = signal(String::new());
    let (edit_pot_medium, set_edit_pot_medium) = signal(String::new());
    let (edit_pot_size, set_edit_pot_size) = signal(String::new());
    let (edit_pot_type, set_edit_pot_type) = signal(String::new());
    let (edit_par_ppfd, set_edit_par_ppfd) = signal(String::new());
    let (edit_rest_start, set_edit_rest_start) = signal(String::new());
    let (edit_rest_end, set_edit_rest_end) = signal(String::new());
    let (edit_bloom_start, set_edit_bloom_start) = signal(String::new());
    let (edit_bloom_end, set_edit_bloom_end) = signal(String::new());
    let (edit_rest_water_mult, set_edit_rest_water_mult) = signal(String::new());
    let (edit_rest_fert_mult, set_edit_rest_fert_mult) = signal(String::new());
    let (edit_active_water_mult, set_edit_active_water_mult) = signal(String::new());
    let (edit_active_fert_mult, set_edit_active_fert_mult) = signal(String::new());

    let populate_edit_fields = move || {
        let current = orchid_signal.get();
        set_edit_name.set(current.name);
        set_edit_species.set(current.species);
        set_edit_water_freq.set(current.water_frequency_days.to_string());
        set_edit_light_req.set(light_req_to_key(&current.light_requirement));
        set_edit_placement.set(current.placement);
        set_edit_light_lux.set(current.light_lux);
        set_edit_temp_range.set(current.temperature_range);
        set_edit_notes.set(current.notes);
        set_edit_conservation.set(current.conservation_status.unwrap_or_default());
        set_edit_temp_min.set(current.temp_min.map(|v| v.to_string()).unwrap_or_default());
        set_edit_temp_max.set(current.temp_max.map(|v| v.to_string()).unwrap_or_default());
        set_edit_humidity_min.set(current.humidity_min.map(|v| v.to_string()).unwrap_or_default());
        set_edit_humidity_max.set(current.humidity_max.map(|v| v.to_string()).unwrap_or_default());
        set_edit_fert_freq.set(current.fertilize_frequency_days.map(|v| v.to_string()).unwrap_or_default());
        set_edit_fert_type.set(current.fertilizer_type.unwrap_or_default());
        set_edit_pot_medium.set(current.pot_medium.map(|v| serde_variant_name(&v)).unwrap_or_default());
        set_edit_pot_size.set(current.pot_size.map(|v| serde_variant_name(&v)).unwrap_or_default());
        set_edit_pot_type.set(current.pot_type.map(|v| serde_variant_name(&v)).unwrap_or_default());
        set_edit_par_ppfd.set(current.par_ppfd.map(|v| v.to_string()).unwrap_or_default());
        set_edit_rest_start.set(current.rest_start_month.map(|v| v.to_string()).unwrap_or_default());
        set_edit_rest_end.set(current.rest_end_month.map(|v| v.to_string()).unwrap_or_default());
        set_edit_bloom_start.set(current.bloom_start_month.map(|v| v.to_string()).unwrap_or_default());
        set_edit_bloom_end.set(current.bloom_end_month.map(|v| v.to_string()).unwrap_or_default());
        set_edit_rest_water_mult.set(current.rest_water_multiplier.map(|v| v.to_string()).unwrap_or_default());
        set_edit_rest_fert_mult.set(current.rest_fertilizer_multiplier.map(|v| v.to_string()).unwrap_or_default());
        set_edit_active_water_mult.set(current.active_water_multiplier.map(|v| v.to_string()).unwrap_or_default());
        set_edit_active_fert_mult.set(current.active_fertilizer_multiplier.map(|v| v.to_string()).unwrap_or_default());
    };

    let on_edit_save = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let current = orchid_signal.get();
        let light_req = match edit_light_req.get().as_str() {
            "Low" => LightRequirement::Low,
            "High" => LightRequirement::High,
            _ => LightRequirement::Medium,
        };
        let cons = edit_conservation.get();
        let conservation_opt = if cons.is_empty() { None } else { Some(cons) };
        let fert_type_val = edit_fert_type.get();
        let pot_medium_val = edit_pot_medium.get();
        let pot_size_val = edit_pot_size.get();
        let pot_type_val = edit_pot_type.get();
        let updated = Orchid {
            id: current.id,
            name: edit_name.get(),
            species: edit_species.get(),
            water_frequency_days: edit_water_freq.get().parse().unwrap_or(7),
            light_requirement: light_req,
            notes: edit_notes.get(),
            placement: edit_placement.get(),
            light_lux: edit_light_lux.get(),
            temperature_range: edit_temp_range.get(),
            conservation_status: conservation_opt,
            native_region: current.native_region,
            native_latitude: current.native_latitude,
            native_longitude: current.native_longitude,
            last_watered_at: current.last_watered_at,
            temp_min: edit_temp_min.get().parse().ok(),
            temp_max: edit_temp_max.get().parse().ok(),
            humidity_min: edit_humidity_min.get().parse().ok(),
            humidity_max: edit_humidity_max.get().parse().ok(),
            first_bloom_at: current.first_bloom_at,
            last_fertilized_at: current.last_fertilized_at,
            fertilize_frequency_days: edit_fert_freq.get().parse().ok(),
            fertilizer_type: if fert_type_val.is_empty() { None } else { Some(fert_type_val) },
            last_repotted_at: current.last_repotted_at,
            pot_medium: if pot_medium_val.is_empty() { None } else { serde_json::from_str(&format!("\"{}\"", pot_medium_val)).ok() },
            pot_size: if pot_size_val.is_empty() { None } else { serde_json::from_str(&format!("\"{}\"", pot_size_val)).ok() },
            pot_type: if pot_type_val.is_empty() { None } else { serde_json::from_str(&format!("\"{}\"", pot_type_val)).ok() },
            par_ppfd: edit_par_ppfd.get().parse().ok(),
            rest_start_month: edit_rest_start.get().parse().ok(),
            rest_end_month: edit_rest_end.get().parse().ok(),
            bloom_start_month: edit_bloom_start.get().parse().ok(),
            bloom_end_month: edit_bloom_end.get().parse().ok(),
            rest_water_multiplier: edit_rest_water_mult.get().parse().ok(),
            rest_fertilizer_multiplier: edit_rest_fert_mult.get().parse().ok(),
            active_water_multiplier: edit_active_water_mult.get().parse().ok(),
            active_fertilizer_multiplier: edit_active_fert_mult.get().parse().ok(),
        };
        set_orchid_signal.set(updated.clone());
        on_update(updated);
        set_is_editing.set(false);
    };

    let on_edit_cancel = move |_| {
        populate_edit_fields();
        set_is_editing.set(false);
    };

    view! {
        // View/Edit toggle
        {move || {
            if !read_only && is_editing.get() {
                let zones_ref = zones.get_value();
                view! {
                    <EditForm
                        edit_name=edit_name set_edit_name=set_edit_name
                        edit_species=edit_species set_edit_species=set_edit_species
                        edit_water_freq=edit_water_freq set_edit_water_freq=set_edit_water_freq
                        edit_light_req=edit_light_req set_edit_light_req=set_edit_light_req
                        edit_placement=edit_placement set_edit_placement=set_edit_placement
                        edit_light_lux=edit_light_lux set_edit_light_lux=set_edit_light_lux
                        edit_temp_range=edit_temp_range set_edit_temp_range=set_edit_temp_range
                        edit_notes=edit_notes set_edit_notes=set_edit_notes
                        edit_conservation=edit_conservation set_edit_conservation=set_edit_conservation
                        edit_temp_min=edit_temp_min set_edit_temp_min=set_edit_temp_min
                        edit_temp_max=edit_temp_max set_edit_temp_max=set_edit_temp_max
                        edit_humidity_min=edit_humidity_min set_edit_humidity_min=set_edit_humidity_min
                        edit_humidity_max=edit_humidity_max set_edit_humidity_max=set_edit_humidity_max
                        edit_fert_freq=edit_fert_freq set_edit_fert_freq=set_edit_fert_freq
                        edit_fert_type=edit_fert_type set_edit_fert_type=set_edit_fert_type
                        edit_pot_medium=edit_pot_medium set_edit_pot_medium=set_edit_pot_medium
                        edit_pot_size=edit_pot_size set_edit_pot_size=set_edit_pot_size
                        edit_pot_type=edit_pot_type set_edit_pot_type=set_edit_pot_type
                        edit_par_ppfd=edit_par_ppfd set_edit_par_ppfd=set_edit_par_ppfd
                        edit_rest_start=edit_rest_start set_edit_rest_start=set_edit_rest_start
                        edit_rest_end=edit_rest_end set_edit_rest_end=set_edit_rest_end
                        edit_bloom_start=edit_bloom_start set_edit_bloom_start=set_edit_bloom_start
                        edit_bloom_end=edit_bloom_end set_edit_bloom_end=set_edit_bloom_end
                        edit_rest_water_mult=edit_rest_water_mult set_edit_rest_water_mult=set_edit_rest_water_mult
                        edit_rest_fert_mult=edit_rest_fert_mult set_edit_rest_fert_mult=set_edit_rest_fert_mult
                        edit_active_water_mult=edit_active_water_mult set_edit_active_water_mult=set_edit_active_water_mult
                        edit_active_fert_mult=edit_active_fert_mult set_edit_active_fert_mult=set_edit_active_fert_mult
                        zones=zones_ref
                        on_save=on_edit_save
                        on_cancel=on_edit_cancel
                    />
                }.into_any()
            } else {
                view! {
                    <div class="mb-4">
                        <div class="flex justify-between items-center mb-3">
                            <h3 class="m-0">"Plant Info"</h3>
                            {(!read_only).then(|| view! {
                                <button class=EDIT_BTN on:click=move |_| {
                                    populate_edit_fields();
                                    set_is_editing.set(true);
                                }>"Edit"</button>
                            })}
                        </div>
                        {move || orchid_signal.get().conservation_status.map(|status| {
                            view! { <p class="my-1 text-sm"><span class="inline-block py-0.5 px-2 text-xs font-medium rounded-full border text-danger bg-danger/5 border-danger/20">{status}</span></p> }
                        })}
                        <div class="grid grid-cols-2 gap-3 text-sm">
                            <div>
                                <div class="text-xs text-stone-400">"Light"</div>
                                <div class="font-medium text-stone-700 dark:text-stone-300">{move || orchid_signal.get().light_requirement.to_string()}</div>
                            </div>
                            <div>
                                <div class="text-xs text-stone-400">"Zone"</div>
                                <div class="font-medium text-stone-700 dark:text-stone-300">{move || orchid_signal.get().placement.clone()}</div>
                            </div>
                            {move || orchid_signal.get().par_ppfd.map(|ppfd| {
                                view! {
                                    <div>
                                        <div class="text-xs text-stone-400">"PAR (PPFD)"</div>
                                        <div class="font-medium text-stone-700 dark:text-stone-300">
                                            {format!("{} \u{00B5}mol/m\u{00B2}/s", ppfd as u32)}
                                        </div>
                                    </div>
                                }
                            })}
                            <div>
                                <div class="text-xs text-stone-400">"Water Every"</div>
                                <div class="font-medium text-stone-700 dark:text-stone-300">{move || {
                                    let o = orchid_signal.get();
                                    let hemi = Hemisphere::from_code(&hemisphere.get_value());
                                    let snap = climate_snapshot.get_value();
                                    let estimate = o.climate_adjusted_water_frequency(&hemi, snap.as_ref());
                                    if estimate.climate_active {
                                        format!("~{} days (base: {})", estimate.adjusted_days, o.water_frequency_days)
                                    } else {
                                        format!("{} days", o.water_frequency_days)
                                    }
                                }}</div>
                            </div>
                            <div>
                                <div class="text-xs text-stone-400">"Temp Range"</div>
                                <div class="font-medium text-stone-700 dark:text-stone-300">{move || orchid_signal.get().temperature_range.clone()}</div>
                            </div>
                        </div>
                        {move || {
                            let notes = orchid_signal.get().notes.clone();
                            (!notes.is_empty()).then(|| {
                                view! { <p class="mt-3 text-sm text-stone-600 dark:text-stone-400">{notes}</p> }
                            })
                        }}
                    </div>
                }.into_any()
            }
        }}

        // Care Schedule: Fertilizer + Pot Info
        <CareScheduleCard orchid_signal=orchid_signal set_orchid_signal=set_orchid_signal read_only=read_only />
        
        // Suitability (Scientific Setup Check)
        {move || {
            let snap = climate_snapshot.get_value();
            view! {
                <crate::components::suitability_card::SuitabilityCard orchid_signal=orchid_signal climate_snapshot=snap />
            }
        }}

        // Seasonal care
        <SeasonalCareCard orchid_signal=orchid_signal hemisphere=hemisphere />

        // Habitat weather
        {native_lat.zip(native_lon).map(|(lat, lon)| {
            let region = native_region.get_value().unwrap_or_else(|| "Native habitat".to_string());
            let zr = habitat_zone_reading.get_value();
            view! {
                <HabitatWeatherCard
                    native_region=region
                    latitude=lat
                    longitude=lon
                    zone_reading=zr
                />
            }
        })}

        // Watering status + Water Now button
        <div class="flex gap-3 justify-between items-center p-4 mt-4 mb-4 rounded-xl bg-secondary">
            <div>
                <div class="text-xs tracking-wide text-stone-400">"Watering Status"</div>
                <div class="text-sm font-medium text-stone-700 dark:text-stone-300">
                    {move || {
                        let o = orchid_signal.get();
                        let hemi = Hemisphere::from_code(&hemisphere.get_value());
                        let snap = climate_snapshot.get_value();
                        let estimate = o.climate_adjusted_water_frequency(&hemi, snap.as_ref());
                        let climate_active = estimate.climate_active;
                        let approx = if climate_active { "~" } else { "" };
                        match o.climate_days_until_due(&hemi, snap.as_ref()) {
                            Some(days) if days < 0 => format!("Overdue by {}{} days", approx, -days),
                            Some(0) => "Due today".to_string(),
                            Some(1) => "Due tomorrow".to_string(),
                            Some(days) => format!("Due in {}{} days", approx, days),
                            None => "Never watered".to_string(),
                        }
                    }}
                </div>
            </div>
            {(!read_only).then(|| view! {
                <button
                    class=BTN_PRIMARY
                    disabled=move || is_watering.get()
                    on:click=move |_| {
                        set_is_watering.set(true);
                        let orchid_id = orchid_signal.get().id.clone();
                        let orchid_id_for_log = orchid_id.clone();
                        leptos::task::spawn_local(async move {
                            match crate::server_fns::orchids::mark_watered(orchid_id).await {
                                Ok(updated) => {
                                    set_orchid_signal.set(updated);
                                    // Refresh journal so the watering entry appears
                                    if let Ok(entries) = crate::server_fns::orchids::get_log_entries(orchid_id_for_log).await {
                                        set_log_entries.set(entries);
                                    }
                                }
                                Err(e) => tracing::error!("Failed to mark watered: {}", e),
                            }
                            set_is_watering.set(false);
                        });
                    }
                >
                    {move || if is_watering.get() { "Watering..." } else { "Water Now" }}
                </button>
            })}
        </div>
    }.into_any()
}

// ── Edit Form sub-component ──────────────────────────────────────────

// ── Care Schedule Card ───────────────────────────────────────────────

const CARE_CARD: &str = "p-4 mb-4 rounded-xl border border-stone-200 dark:border-stone-700";
const CARE_STAT_LABEL: &str = "text-xs tracking-wide text-stone-400";
const CARE_STAT_VALUE: &str = "text-sm font-medium text-stone-700 dark:text-stone-300";

#[component]
fn CareScheduleCard(
    orchid_signal: ReadSignal<Orchid>,
    set_orchid_signal: WriteSignal<Orchid>,
    #[prop(optional)] read_only: bool,
) -> impl IntoView {
    let (is_fertilizing, set_is_fertilizing) = signal(false);

    view! {
        <div class=CARE_CARD>
            <h3 class="mt-0 mb-3 text-sm font-semibold tracking-wide text-stone-500 dark:text-stone-400">"Care Schedule"</h3>

            <div class="grid grid-cols-2 gap-3 text-sm">
                // Fertilizer section
                <div>
                    <div class=CARE_STAT_LABEL>"\u{2728} Fertilizer"</div>
                    <div class=CARE_STAT_VALUE>
                        {move || {
                            let o = orchid_signal.get();
                            o.fertilizer_type.unwrap_or_else(|| "Not set".to_string())
                        }}
                    </div>
                </div>
                <div>
                    <div class=CARE_STAT_LABEL>"Fertilize Every"</div>
                    <div class=CARE_STAT_VALUE>
                        {move || {
                            let o = orchid_signal.get();
                            match o.fertilize_frequency_days {
                                Some(d) => format!("{} days", d),
                                None => "No schedule".to_string(),
                            }
                        }}
                    </div>
                </div>
                <div>
                    <div class=CARE_STAT_LABEL>"Last Fertilized"</div>
                    <div class={move || {
                        let o = orchid_signal.get();
                        let overdue = o.fertilize_frequency_days.is_some()
                            && o.fertilize_days_until_due().map(|d| d < 0).unwrap_or(false);
                        if overdue { "text-sm font-medium text-danger" } else { CARE_STAT_VALUE }
                    }}>
                        {move || {
                            let o = orchid_signal.get();
                            match o.days_since_fertilized() {
                                Some(0) => "Today".to_string(),
                                Some(1) => "1 day ago".to_string(),
                                Some(d) => format!("{} days ago", d),
                                None => "Never".to_string(),
                            }
                        }}
                    </div>
                </div>
                {(!read_only).then(|| view! {
                    <div class="flex items-end">
                        <button
                            class="py-1.5 px-3 text-xs font-semibold text-yellow-700 bg-yellow-100 rounded-lg border-none transition-colors cursor-pointer dark:text-yellow-300 hover:bg-yellow-200 dark:bg-yellow-900/30 dark:hover:bg-yellow-900/50"
                            disabled=move || is_fertilizing.get()
                            on:click=move |_| {
                                set_is_fertilizing.set(true);
                                let orchid_id = orchid_signal.get().id.clone();
                                leptos::task::spawn_local(async move {
                                    match crate::server_fns::orchids::mark_fertilized(orchid_id).await {
                                        Ok(updated) => set_orchid_signal.set(updated),
                                        Err(e) => tracing::error!("Failed to mark fertilized: {}", e),
                                    }
                                    set_is_fertilizing.set(false);
                                });
                            }
                        >
                            {move || if is_fertilizing.get() { "..." } else { "\u{2728} Fertilize" }}
                        </button>
                    </div>
                })}
            </div>

            // Pot info
            <div class="grid grid-cols-2 gap-3 pt-3 mt-3 text-sm border-t border-stone-100 dark:border-stone-700/50">
                {move || {
                    let is_mounted = orchid_signal.get().pot_type.as_ref() == Some(&crate::orchid::PotType::Mounted);
                    (!is_mounted).then(|| view! {
                        <div>
                            <div class=CARE_STAT_LABEL>"\u{1FAB4} Pot Medium"</div>
                            <div class=CARE_STAT_VALUE>
                                {orchid_signal.get().pot_medium.map(|v| v.to_string()).unwrap_or_else(|| "Not set".to_string())}
                            </div>
                        </div>
                    })
                }}
                <div>
                    <div class=CARE_STAT_LABEL>"Pot Type"</div>
                    <div class=CARE_STAT_VALUE>
                        {move || orchid_signal.get().pot_type.map(|v| v.to_string()).unwrap_or_else(|| "Not set".to_string())}
                    </div>
                </div>
                {move || {
                    let is_mounted = orchid_signal.get().pot_type.as_ref() == Some(&crate::orchid::PotType::Mounted);
                    (!is_mounted).then(|| view! {
                        <div>
                            <div class=CARE_STAT_LABEL>"Pot Size"</div>
                            <div class=CARE_STAT_VALUE>
                                {orchid_signal.get().pot_size.map(|v| v.to_string()).unwrap_or_else(|| "Not set".to_string())}
                            </div>
                        </div>
                    })
                }}
                <div>
                    <div class=CARE_STAT_LABEL>"Last Repotted"</div>
                    <div class=CARE_STAT_VALUE>
                        {move || {
                            let o = orchid_signal.get();
                            match o.days_since_repotted() {
                                Some(0) => "Today".to_string(),
                                Some(d) if d < 30 => format!("{} days ago", d),
                                Some(d) if d < 365 => format!("{} months ago", d / 30),
                                Some(d) => format!("{:.1} years ago", d as f64 / 365.0),
                                None => "Never".to_string(),
                            }
                        }}
                    </div>
                </div>
            </div>
        </div>
    }.into_any()
}

// ── Seasonal Care Card ───────────────────────────────────────────────

#[component]
fn SeasonalCareCard(
    orchid_signal: ReadSignal<Orchid>,
    hemisphere: StoredValue<String>,
) -> impl IntoView {
    let hemi = Hemisphere::from_code(&hemisphere.get_value());
    let hemi_for_bar = hemi.clone();
    let hemi_for_freq = hemi.clone();
    let hemi_for_next = hemi.clone();

    view! {
        {move || {
            let o = orchid_signal.get();
            if !o.has_seasonal_data() {
                return view! { <div></div> }.into_any();
            }

            let phase = o.current_phase(&hemi);
            let (badge_class, badge_text) = match &phase {
                SeasonalPhase::Rest => (
                    "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300",
                    "Rest Period"
                ),
                SeasonalPhase::Blooming => (
                    "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-pink-100 text-pink-700 dark:bg-pink-900/30 dark:text-pink-300",
                    "Blooming"
                ),
                SeasonalPhase::Active => (
                    "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300",
                    "Active Growth"
                ),
                SeasonalPhase::Unknown => (
                    "inline-flex py-0.5 px-2 text-xs font-semibold rounded-full bg-stone-100 text-stone-600 dark:bg-stone-800 dark:text-stone-400",
                    "Unknown"
                ),
            };

            let eff_water = o.effective_water_frequency(&hemi_for_freq);
            let eff_fert = o.effective_fertilize_frequency(&hemi_for_freq);

            let next_transition = o.next_transition(&hemi_for_next);

            // Build the 12-month bar
            let now_month = chrono::Utc::now().month();
            let month_cells = (1..=12u32).map(|m| {
                let in_rest = o.rest_start_month.zip(o.rest_end_month)
                    .map(|(s, e)| month_in_range(m, hemi_for_bar.adjust_month(s), hemi_for_bar.adjust_month(e)))
                    .unwrap_or(false);
                let in_bloom = o.bloom_start_month.zip(o.bloom_end_month)
                    .map(|(s, e)| month_in_range(m, hemi_for_bar.adjust_month(s), hemi_for_bar.adjust_month(e)))
                    .unwrap_or(false);
                let is_current = m == now_month;

                let bg = if in_bloom {
                    "bg-pink-200 dark:bg-pink-900/40"
                } else if in_rest {
                    "bg-blue-200 dark:bg-blue-900/40"
                } else {
                    "bg-emerald-100 dark:bg-emerald-900/30"
                };

                let border = if is_current { " ring-2 ring-primary ring-offset-1" } else { "" };
                let class = format!("flex flex-col justify-center items-center py-1 rounded text-[10px] {}{}", bg, border);

                view! {
                    <div class=class>
                        <span class="font-medium">{Orchid::month_name(m)}</span>
                    </div>
                }
            }).collect::<Vec<_>>();

            view! {
                <div class=CARE_CARD>
                    <div class="flex gap-2 justify-between items-center mb-3">
                        <h3 class="m-0 text-sm font-semibold tracking-wide text-stone-500 dark:text-stone-400">"Seasonal Care"</h3>
                        <span class=badge_class>{badge_text}</span>
                    </div>

                    // 12-month bar
                    <div class="grid grid-cols-12 gap-0.5 mb-3">
                        {month_cells}
                    </div>

                    // Legend
                    <div class="flex gap-3 mb-3 text-xs text-stone-400">
                        <span class="flex gap-1 items-center"><span class="inline-block w-3 h-3 bg-blue-200 rounded dark:bg-blue-900/40"></span>"Rest"</span>
                        <span class="flex gap-1 items-center"><span class="inline-block w-3 h-3 bg-pink-200 rounded dark:bg-pink-900/40"></span>"Bloom"</span>
                        <span class="flex gap-1 items-center"><span class="inline-block w-3 h-3 bg-emerald-100 rounded dark:bg-emerald-900/30"></span>"Active"</span>
                    </div>

                    // Effective frequencies
                    <div class="grid grid-cols-2 gap-3 text-sm">
                        <div>
                            <div class=CARE_STAT_LABEL>"Effective Water Freq"</div>
                            <div class=CARE_STAT_VALUE>{format!("Every {} days", eff_water)}</div>
                        </div>
                        <div>
                            <div class=CARE_STAT_LABEL>"Effective Fert Freq"</div>
                            <div class=CARE_STAT_VALUE>
                                {match eff_fert {
                                    Some(d) => format!("Every {} days", d),
                                    None => "No schedule".to_string(),
                                }}
                            </div>
                        </div>
                    </div>

                    // Next transition
                    {next_transition.map(|(month, label)| {
                        view! {
                            <div class="pt-2 mt-2 text-xs border-t text-stone-400 border-stone-100 dark:border-stone-700/50">
                                {format!("{}: {}", label, Orchid::month_name(month))}
                            </div>
                        }
                    })}
                </div>
            }.into_any()
        }}
    }.into_any()
}

// ── Edit Form sub-component ──────────────────────────────────────────

#[component]
fn EditForm(
    edit_name: ReadSignal<String>, set_edit_name: WriteSignal<String>,
    edit_species: ReadSignal<String>, set_edit_species: WriteSignal<String>,
    edit_water_freq: ReadSignal<String>, set_edit_water_freq: WriteSignal<String>,
    edit_light_req: ReadSignal<String>, set_edit_light_req: WriteSignal<String>,
    edit_placement: ReadSignal<String>, set_edit_placement: WriteSignal<String>,
    edit_light_lux: ReadSignal<String>, set_edit_light_lux: WriteSignal<String>,
    edit_temp_range: ReadSignal<String>, set_edit_temp_range: WriteSignal<String>,
    edit_notes: ReadSignal<String>, set_edit_notes: WriteSignal<String>,
    edit_conservation: ReadSignal<String>, set_edit_conservation: WriteSignal<String>,
    edit_temp_min: ReadSignal<String>, set_edit_temp_min: WriteSignal<String>,
    edit_temp_max: ReadSignal<String>, set_edit_temp_max: WriteSignal<String>,
    edit_humidity_min: ReadSignal<String>, set_edit_humidity_min: WriteSignal<String>,
    edit_humidity_max: ReadSignal<String>, set_edit_humidity_max: WriteSignal<String>,
    edit_fert_freq: ReadSignal<String>, set_edit_fert_freq: WriteSignal<String>,
    edit_fert_type: ReadSignal<String>, set_edit_fert_type: WriteSignal<String>,
    edit_pot_medium: ReadSignal<String>, set_edit_pot_medium: WriteSignal<String>,
    edit_pot_size: ReadSignal<String>, set_edit_pot_size: WriteSignal<String>,
    edit_pot_type: ReadSignal<String>, set_edit_pot_type: WriteSignal<String>,
    edit_par_ppfd: ReadSignal<String>, set_edit_par_ppfd: WriteSignal<String>,
    edit_rest_start: ReadSignal<String>, set_edit_rest_start: WriteSignal<String>,
    edit_rest_end: ReadSignal<String>, set_edit_rest_end: WriteSignal<String>,
    edit_bloom_start: ReadSignal<String>, set_edit_bloom_start: WriteSignal<String>,
    edit_bloom_end: ReadSignal<String>, set_edit_bloom_end: WriteSignal<String>,
    edit_rest_water_mult: ReadSignal<String>, set_edit_rest_water_mult: WriteSignal<String>,
    edit_rest_fert_mult: ReadSignal<String>, set_edit_rest_fert_mult: WriteSignal<String>,
    edit_active_water_mult: ReadSignal<String>, set_edit_active_water_mult: WriteSignal<String>,
    edit_active_fert_mult: ReadSignal<String>, set_edit_active_fert_mult: WriteSignal<String>,
    zones: Vec<GrowingZone>,
    on_save: impl Fn(leptos::ev::SubmitEvent) + 'static + Copy + Send + Sync,
    on_cancel: impl Fn(leptos::ev::MouseEvent) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let on_auto_calculate = move |_ev: leptos::ev::MouseEvent| {
        let size = serde_json::from_str::<crate::orchid::PotSize>(&format!("\"{}\"", edit_pot_size.get())).unwrap_or_default();
        let medium = serde_json::from_str::<crate::orchid::PotMedium>(&format!("\"{}\"", edit_pot_medium.get())).unwrap_or_default();
        let p_type = serde_json::from_str::<crate::orchid::PotType>(&format!("\"{}\"", edit_pot_type.get())).unwrap_or_default();
        
        let light_req = match edit_light_req.get().as_str() {
            "Low" => LightRequirement::Low,
            "High" => LightRequirement::High,
            _ => LightRequirement::Medium,
        };

        let par: Option<f64> = edit_par_ppfd.get().parse().ok();

        let days = crate::estimation::calculate_algorithmic_base_days(
            &size,
            &medium,
            &p_type,
            &light_req,
            crate::estimation::VPD_BASELINE, // Using baseline indoors
            par,
        );
        set_edit_water_freq.set(days.to_string());
    };

    view! {
        <div class="mb-6">
            <form on:submit=on_save>
                <div class="mb-4">
                    <label>"Name:"</label>
                    <input type="text" prop:value=edit_name on:input=move |ev| set_edit_name.set(event_target_value(&ev)) required />
                </div>
                <div class="mb-4">
                    <label>"Species:"</label>
                    <input type="text" prop:value=edit_species on:input=move |ev| set_edit_species.set(event_target_value(&ev)) required />
                </div>
                <div class="mb-4">
                    <label>"Conservation Status:"</label>
                    <input type="text" prop:value=edit_conservation on:input=move |ev| set_edit_conservation.set(event_target_value(&ev)) placeholder="e.g. CITES II (optional)" />
                </div>
                <div class="flex flex-col gap-4 mb-4 sm:flex-row">
                    <div class="flex-1">
                        <div class="flex justify-between items-center">
                            <label>"Water Freq (days):"</label>
                            <button 
                                type="button" 
                                class="transition-colors focus:outline-none text-[10px] text-primary hover:text-primary-light"
                                on:click=on_auto_calculate
                                title="Auto-calculate based on pot size, medium, and type"
                            >
                                "\u{2728} Auto-Calc"
                            </button>
                        </div>
                        <input type="number" prop:value=edit_water_freq on:input=move |ev| set_edit_water_freq.set(event_target_value(&ev)) required />
                    </div>
                    <div class="flex-1">
                        <label>"Light Req:"</label>
                        <select prop:value=edit_light_req on:change=move |ev| set_edit_light_req.set(event_target_value(&ev))>
                            <option value="Low">"Low"</option>
                            <option value="Medium">"Medium"</option>
                            <option value="High">"High"</option>
                        </select>
                    </div>
                </div>
                <div class="flex flex-col gap-4 mb-4 sm:flex-row">
                    <div class="flex-1">
                        <label>"Zone:"</label>
                        <select prop:value=edit_placement on:change=move |ev| set_edit_placement.set(event_target_value(&ev))>
                            {zones.iter().map(|zone| {
                                let name = zone.name.clone();
                                let label = format!("{} ({})", zone.name, zone.light_level);
                                view! { <option value=name>{label}</option> }
                            }).collect::<Vec<_>>()}
                        </select>
                    </div>
                    <div class="flex-1">
                        <label>"Light (Lux):"</label>
                        <input type="text" prop:value=edit_light_lux on:input=move |ev| set_edit_light_lux.set(event_target_value(&ev)) placeholder="e.g. 5000" />
                    </div>
                    <div class="flex-1">
                        <label>"PAR (PPFD):"</label>
                        <input type="number" step="1" min="0" max="2500" prop:value=edit_par_ppfd on:input=move |ev| set_edit_par_ppfd.set(event_target_value(&ev)) placeholder="\u{00B5}mol/m\u{00B2}/s" />
                    </div>
                </div>
                <div class="mb-4">
                    <label>"Temp Range:"</label>
                    <input type="text" prop:value=edit_temp_range on:input=move |ev| set_edit_temp_range.set(event_target_value(&ev)) placeholder="e.g. 18-28C" />
                </div>
                <div class="flex flex-col gap-4 mb-4 sm:flex-row">
                    <div class="flex-1">
                        <label>"Min Temp (C):"</label>
                        <input type="number" step="0.1" prop:value=edit_temp_min on:input=move |ev| set_edit_temp_min.set(event_target_value(&ev)) placeholder="e.g. 18" />
                    </div>
                    <div class="flex-1">
                        <label>"Max Temp (C):"</label>
                        <input type="number" step="0.1" prop:value=edit_temp_max on:input=move |ev| set_edit_temp_max.set(event_target_value(&ev)) placeholder="e.g. 28" />
                    </div>
                </div>
                <div class="flex flex-col gap-4 mb-4 sm:flex-row">
                    <div class="flex-1">
                        <label>"Min Humidity (%):"</label>
                        <input type="number" step="0.1" prop:value=edit_humidity_min on:input=move |ev| set_edit_humidity_min.set(event_target_value(&ev)) placeholder="e.g. 50" />
                    </div>
                    <div class="flex-1">
                        <label>"Max Humidity (%):"</label>
                        <input type="number" step="0.1" prop:value=edit_humidity_max on:input=move |ev| set_edit_humidity_max.set(event_target_value(&ev)) placeholder="e.g. 80" />
                    </div>
                </div>

                // ── Fertilizer & Pot Section ──
                <div class="pt-4 mt-4 border-t border-stone-200 dark:border-stone-700">
                    <h4 class="mt-0 mb-3 text-xs font-semibold tracking-widest uppercase text-stone-500 dark:text-stone-400">"Fertilizer & Pot"</h4>
                    <div class="flex flex-col gap-4 mb-4 sm:flex-row">
                        <div class="flex-1">
                            <label>"Fertilizer Type:"</label>
                            <input type="text" prop:value=edit_fert_type on:input=move |ev| set_edit_fert_type.set(event_target_value(&ev)) placeholder="e.g. MSU, Bloom Booster" />
                        </div>
                        <div class="flex-1">
                            <label>"Fertilize Every (days):"</label>
                            <input type="number" prop:value=edit_fert_freq on:input=move |ev| set_edit_fert_freq.set(event_target_value(&ev)) placeholder="e.g. 14" />
                        </div>
                    </div>
                    <div class="flex flex-col gap-4 mb-4 sm:flex-row">
                        {move || (edit_pot_type.get() != "Mounted").then(|| view! {
                            <div class="flex-1 animate-fade-in">
                                <label>"Pot Medium:"</label>
                                <select prop:value=edit_pot_medium on:change=move |ev| set_edit_pot_medium.set(event_target_value(&ev))>
                                    <option value="">"Unknown / Unset"</option>
                                    <option value="Bark">"Bark"</option>
                                    <option value="Sphagnum Moss">"Sphagnum Moss"</option>
                                    <option value="LECA">"LECA (Semi-Hydro)"</option>
                                    <option value="Inorganic">"Inorganic (Lava/Pumice)"</option>
                                </select>
                            </div>
                        })}
                        <div class="flex-1">
                            <label>"Pot Type (Airflow):"</label>
                            <select prop:value=edit_pot_type on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_edit_pot_type.set(val.clone());
                                if val == "Mounted" {
                                    set_edit_pot_medium.set(String::new());
                                    set_edit_pot_size.set(String::new());
                                }
                            }>
                                <option value="">"Unknown / Unset"</option>
                                <option value="Solid">"Solid (Plastic/Glazed)"</option>
                                <option value="Slotted">"Slotted (Net Pot)"</option>
                                <option value="Clay">"Terra Cotta (Clay)"</option>
                                <option value="Mounted">"Mounted (Slab)"</option>
                            </select>
                        </div>
                        {move || (edit_pot_type.get() != "Mounted").then(|| view! {
                            <div class="flex-1 animate-fade-in">
                                <label>"Pot Size:"</label>
                                <select prop:value=edit_pot_size on:change=move |ev| set_edit_pot_size.set(event_target_value(&ev))>
                                    <option value="">"Unknown / Unset"</option>
                                    <option value="Small">"Small (2-3\")"</option>
                                    <option value="Medium">"Medium (4-5\")"</option>
                                    <option value="Large">"Large (6\"+)"</option>
                                </select>
                            </div>
                        })}
                    </div>
                </div>

                // ── Seasonal Care Section ──
                <div class="pt-4 mt-4 border-t border-stone-200 dark:border-stone-700">
                    <h4 class="mt-0 mb-3 text-xs font-semibold tracking-widest uppercase text-stone-500 dark:text-stone-400">"Seasonal Care"</h4>
                    <div class="flex flex-col gap-4 mb-4 sm:flex-row">
                        <div class="flex-1">
                            <label>"Rest Start Month (1-12):"</label>
                            <input type="number" min="1" max="12" prop:value=edit_rest_start on:input=move |ev| set_edit_rest_start.set(event_target_value(&ev)) placeholder="e.g. 11" />
                        </div>
                        <div class="flex-1">
                            <label>"Rest End Month (1-12):"</label>
                            <input type="number" min="1" max="12" prop:value=edit_rest_end on:input=move |ev| set_edit_rest_end.set(event_target_value(&ev)) placeholder="e.g. 2" />
                        </div>
                    </div>
                    <div class="flex flex-col gap-4 mb-4 sm:flex-row">
                        <div class="flex-1">
                            <label>"Bloom Start Month (1-12):"</label>
                            <input type="number" min="1" max="12" prop:value=edit_bloom_start on:input=move |ev| set_edit_bloom_start.set(event_target_value(&ev)) placeholder="e.g. 3" />
                        </div>
                        <div class="flex-1">
                            <label>"Bloom End Month (1-12):"</label>
                            <input type="number" min="1" max="12" prop:value=edit_bloom_end on:input=move |ev| set_edit_bloom_end.set(event_target_value(&ev)) placeholder="e.g. 5" />
                        </div>
                    </div>
                    <div class="flex flex-col gap-4 mb-4 sm:flex-row">
                        <div class="flex-1">
                            <label>"Rest Water Mult:"</label>
                            <input type="number" step="0.1" min="0" max="1" prop:value=edit_rest_water_mult on:input=move |ev| set_edit_rest_water_mult.set(event_target_value(&ev)) placeholder="e.g. 0.3" />
                        </div>
                        <div class="flex-1">
                            <label>"Rest Fert Mult:"</label>
                            <input type="number" step="0.1" min="0" max="1" prop:value=edit_rest_fert_mult on:input=move |ev| set_edit_rest_fert_mult.set(event_target_value(&ev)) placeholder="e.g. 0.0" />
                        </div>
                    </div>
                    <div class="flex flex-col gap-4 mb-4 sm:flex-row">
                        <div class="flex-1">
                            <label>"Active Water Mult:"</label>
                            <input type="number" step="0.1" min="0" max="2" prop:value=edit_active_water_mult on:input=move |ev| set_edit_active_water_mult.set(event_target_value(&ev)) placeholder="e.g. 1.0" />
                        </div>
                        <div class="flex-1">
                            <label>"Active Fert Mult:"</label>
                            <input type="number" step="0.1" min="0" max="2" prop:value=edit_active_fert_mult on:input=move |ev| set_edit_active_fert_mult.set(event_target_value(&ev)) placeholder="e.g. 1.0" />
                        </div>
                    </div>
                </div>

                <div class="mb-4">
                    <label>"Notes:"</label>
                    <textarea prop:value=edit_notes on:input=move |ev| set_edit_notes.set(event_target_value(&ev)) rows="3"></textarea>
                </div>
                <div class="flex gap-2">
                    <button type="submit" class=BTN_PRIMARY>"Save"</button>
                    <button type="button" class=BTN_SECONDARY on:click=on_cancel>"Cancel"</button>
                </div>
            </form>
        </div>
    }.into_any()
}

// ── SSR Component Rendering Tests ───────────────────────────────────

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use leptos::reactive::owner::Owner;
    use crate::test_helpers::{test_orchid, test_orchid_mounted, test_orchid_with_care};

    // ── CareScheduleCard ────────────────────────────────────────────

    #[test]
    fn test_care_schedule_card_shows_fertilize_button() {
        let owner = Owner::new();
        owner.with(|| {
            let (orchid_signal, set_orchid_signal) = signal(test_orchid_with_care());
            let html = view! {
                <CareScheduleCard
                    orchid_signal=orchid_signal
                    set_orchid_signal=set_orchid_signal
                    read_only=false
                />
            }.to_html();
            assert!(html.contains("\u{2728} Fertilize"),
                "Fertilize button should be visible when read_only=false");
        });
    }

    #[test]
    fn test_care_schedule_card_hides_fertilize_when_read_only() {
        let owner = Owner::new();
        owner.with(|| {
            let (orchid_signal, set_orchid_signal) = signal(test_orchid_with_care());
            let html = view! {
                <CareScheduleCard
                    orchid_signal=orchid_signal
                    set_orchid_signal=set_orchid_signal
                    read_only=true
                />
            }.to_html();
            // CareScheduleCard has no buttons in read-only mode
            assert!(!html.contains("<button"),
                "Fertilize button should be hidden in read-only mode, got: {html}");
            // The stat labels should still be present
            assert!(html.contains("Fertilize Every"),
                "Stat labels should still be visible in read-only mode");
        });
    }

    #[test]
    fn test_care_schedule_card_always_shows_stats() {
        let owner = Owner::new();
        owner.with(|| {
            for read_only in [true, false] {
                let (orchid_signal, set_orchid_signal) = signal(test_orchid_with_care());
                let html = view! {
                    <CareScheduleCard
                        orchid_signal=orchid_signal
                        set_orchid_signal=set_orchid_signal
                        read_only=read_only
                    />
                }.to_html();
                assert!(html.contains("Last Fertilized"),
                    "Stats should be visible when read_only={read_only}");
                assert!(html.contains("Pot Medium"),
                    "Pot info should be visible when read_only={read_only}");
            }
        });
    }

    #[test]
    fn test_care_schedule_card_shows_care_data() {
        let owner = Owner::new();
        owner.with(|| {
            let (orchid_signal, set_orchid_signal) = signal(test_orchid_with_care());
            let html = view! {
                <CareScheduleCard
                    orchid_signal=orchid_signal
                    set_orchid_signal=set_orchid_signal
                />
            }.to_html();
            assert!(html.contains("MSU"), "Should show fertilizer type");
            assert!(html.contains("14 days"), "Should show fertilize frequency");
            assert!(html.contains("Bark"), "Should show pot medium");
            assert!(html.contains("Medium (4-5"), "Should show pot size");
        });
    }

    #[test]
    fn test_care_schedule_card_shows_defaults_when_no_care_data() {
        let owner = Owner::new();
        owner.with(|| {
            let (orchid_signal, set_orchid_signal) = signal(test_orchid());
            let html = view! {
                <CareScheduleCard
                    orchid_signal=orchid_signal
                    set_orchid_signal=set_orchid_signal
                />
            }.to_html();
            assert!(html.contains("Not set"), "Should show 'Not set' for missing care data");
            assert!(html.contains("No schedule"), "Should show 'No schedule' for missing frequency");
        });
    }

    #[test]
    fn test_care_schedule_card_hides_pot_medium_and_size_for_mounted() {
        let owner = Owner::new();
        owner.with(|| {
            let (orchid_signal, set_orchid_signal) = signal(test_orchid_mounted());
            let html = view! {
                <CareScheduleCard
                    orchid_signal=orchid_signal
                    set_orchid_signal=set_orchid_signal
                />
            }.to_html();
            assert!(html.contains("Mounted"),
                "Should show 'Mounted' pot type. Got: {html}");
            assert!(!html.contains("Pot Medium"),
                "Pot Medium should be hidden for mounted orchids. Got: {html}");
            assert!(!html.contains("Pot Size"),
                "Pot Size should be hidden for mounted orchids. Got: {html}");
            assert!(html.contains("Last Repotted"),
                "Last Repotted should still be visible for mounted orchids");
        });
    }

    #[test]
    fn test_care_schedule_card_shows_pot_medium_and_size_for_potted() {
        let owner = Owner::new();
        owner.with(|| {
            let (orchid_signal, set_orchid_signal) = signal(test_orchid_with_care());
            let html = view! {
                <CareScheduleCard
                    orchid_signal=orchid_signal
                    set_orchid_signal=set_orchid_signal
                />
            }.to_html();
            assert!(html.contains("Pot Medium"),
                "Pot Medium should be visible for potted orchids");
            assert!(html.contains("Pot Size"),
                "Pot Size should be visible for potted orchids");
            assert!(html.contains("Pot Type"),
                "Pot Type should be visible for potted orchids");
        });
    }

    // ── JournalTab ──────────────────────────────────────────────────

    #[test]
    fn test_journal_tab_hides_note_form_when_read_only() {
        let owner = Owner::new();
        owner.with(|| {
            let (orchid_signal, set_orchid_signal) = signal(test_orchid());
            let (log_entries, set_log_entries) = signal(Vec::new());
            let (_, set_show_first_bloom) = signal(false);
            let html = view! {
                <JournalTab
                    orchid_signal=orchid_signal
                    set_orchid_signal=set_orchid_signal
                    log_entries=log_entries
                    set_log_entries=set_log_entries
                    set_show_first_bloom=set_show_first_bloom
                    read_only=true
                />
            }.to_html();
            assert!(!html.contains("Add a detailed note"),
                "Note form should be hidden in read-only mode, got: {html}");
        });
    }

    #[test]
    fn test_journal_tab_shows_note_form() {
        let owner = Owner::new();
        owner.with(|| {
            let (orchid_signal, set_orchid_signal) = signal(test_orchid());
            let (log_entries, set_log_entries) = signal(Vec::new());
            let (_, set_show_first_bloom) = signal(false);
            let html = view! {
                <JournalTab
                    orchid_signal=orchid_signal
                    set_orchid_signal=set_orchid_signal
                    log_entries=log_entries
                    set_log_entries=set_log_entries
                    set_show_first_bloom=set_show_first_bloom
                    read_only=false
                />
            }.to_html();
            assert!(html.contains("Add a detailed note"),
                "Note form should be visible when read_only=false");
        });
    }
}
