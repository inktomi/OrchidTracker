use leptos::prelude::*;
use crate::orchid::{Orchid, LightRequirement, GrowingZone, ClimateReading, LogEntry};
use crate::components::habitat_weather::HabitatWeatherCard;
use crate::components::event_type_picker::EventTypePicker;
use crate::components::photo_capture::PhotoCapture;
use crate::components::growth_thread::GrowthThread;
use crate::components::first_bloom::FirstBloomCelebration;
use crate::components::photo_gallery::PhotoGallery;
use super::{MODAL_OVERLAY, MODAL_CONTENT, MODAL_HEADER, BTN_PRIMARY, BTN_SECONDARY, BTN_CLOSE};

const EDIT_BTN: &str = "py-2 px-3 text-sm font-semibold text-white rounded-lg border-none cursor-pointer bg-accent hover:bg-accent-dark transition-colors";
const TAB_ACTIVE: &str = "py-2 px-4 text-sm font-semibold border-b-2 cursor-pointer transition-colors text-primary border-primary bg-transparent";
const TAB_INACTIVE: &str = "py-2 px-4 text-sm font-medium border-b-2 border-transparent cursor-pointer transition-colors text-stone-400 hover:text-stone-600 bg-transparent dark:hover:text-stone-300";

fn light_req_to_key(lr: &LightRequirement) -> String {
    match lr {
        LightRequirement::Low => "Low".to_string(),
        LightRequirement::Medium => "Medium".to_string(),
        LightRequirement::High => "High".to_string(),
    }
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
    on_close: impl Fn() + 'static + Send + Sync,
    on_update: impl Fn(Orchid) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let (orchid_signal, set_orchid_signal) = signal(orchid.clone());
    let (log_entries, set_log_entries) = signal(Vec::<LogEntry>::new());
    let (active_tab, set_active_tab) = signal(DetailTab::Journal);
    let (show_first_bloom, set_show_first_bloom) = signal(false);

    // Load log entries on mount
    {
        let orchid_id = orchid.id.clone();
        leptos::task::spawn_local(async move {
            match crate::server_fns::orchids::get_log_entries(orchid_id).await {
                Ok(entries) => set_log_entries.set(entries),
                Err(e) => log::error!("Failed to load log entries: {}", e),
            }
        });
    }

    // Edit mode state
    let (is_editing, set_is_editing) = signal(false);
    let zones_stored = StoredValue::new(zones);

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
                        <p class="mt-0.5 mb-0 text-sm italic text-stone-400">{move || orchid_signal.get().species}</p>
                    </div>
                    <div class="flex gap-2">
                        <button class=BTN_CLOSE on:click=move |_| on_close()>"\u{00D7}"</button>
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
                                log_entries=log_entries
                                set_log_entries=set_log_entries
                                set_show_first_bloom=set_show_first_bloom
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
                                on_update=on_update
                                habitat_zone_reading=habitat_zone_reading
                                native_region=native_region
                                native_lat=native_lat
                                native_lon=native_lon
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
    log_entries: ReadSignal<Vec<LogEntry>>,
    set_log_entries: WriteSignal<Vec<LogEntry>>,
    set_show_first_bloom: WriteSignal<bool>,
) -> impl IntoView {
    let (note, set_note) = signal(String::new());
    let (selected_event_type, set_selected_event_type) = signal(Option::<String>::None);
    let (uploaded_filename, set_uploaded_filename) = signal(Option::<String>::None);
    let (is_syncing, set_is_syncing) = signal(false);

    let on_submit_log = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let current_note = note.get();
        let event_type = selected_event_type.get();
        let image = uploaded_filename.get();

        // Require at least a note, photo, or event type
        if current_note.is_empty() && image.is_none() && event_type.is_none() {
            return;
        }

        set_is_syncing.set(true);
        let orchid_id = orchid_signal.get().id.clone();

        leptos::task::spawn_local(async move {
            match crate::server_fns::orchids::add_log_entry(
                orchid_id,
                current_note,
                image,
                event_type,
            ).await {
                Ok(response) => {
                    if response.is_first_bloom {
                        set_show_first_bloom.set(true);
                    }
                    set_log_entries.update(|entries| entries.insert(0, response.entry));
                }
                Err(e) => log::error!("Failed to add log entry: {}", e),
            }
            set_is_syncing.set(false);
            set_note.set(String::new());
            set_selected_event_type.set(None);
            set_uploaded_filename.set(None);
        });
    };

    view! {
        // Add Entry form
        <div class="p-4 mb-6 rounded-xl border border-stone-200 dark:border-stone-700">
            <form on:submit=on_submit_log>
                // Event type picker
                <div class="mb-3">
                    <label class="mb-2">"What happened?"</label>
                    <EventTypePicker
                        selected=selected_event_type
                        on_select=move |val| set_selected_event_type.set(val)
                    />
                </div>

                // Photo upload
                <div class="mb-3">
                    <PhotoCapture
                        on_photo_ready=move |fname| set_uploaded_filename.set(Some(fname))
                    />
                </div>

                // Note textarea
                <div class="mb-3">
                    <textarea
                        prop:value=note
                        on:input=move |ev| set_note.set(event_target_value(&ev))
                        placeholder="Add a note about this moment..."
                        rows="2"
                        class="py-2 px-3 w-full text-sm bg-white rounded-lg border border-stone-300 dark:bg-stone-800 dark:border-stone-600 dark:text-stone-200"
                    ></textarea>
                </div>

                <button type="submit" class=BTN_PRIMARY disabled=move || is_syncing.get()>
                    {move || if is_syncing.get() { "Saving..." } else { "Add Entry" }}
                </button>
            </form>
        </div>

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
    on_update: impl Fn(Orchid) + 'static + Copy + Send + Sync,
    habitat_zone_reading: StoredValue<Option<ClimateReading>>,
    native_region: StoredValue<Option<String>>,
    native_lat: Option<f64>,
    native_lon: Option<f64>,
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
        set_edit_pot_medium.set(current.pot_medium.unwrap_or_default());
        set_edit_pot_size.set(current.pot_size.unwrap_or_default());
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
            pot_medium: if pot_medium_val.is_empty() { None } else { Some(pot_medium_val) },
            pot_size: if pot_size_val.is_empty() { None } else { Some(pot_size_val) },
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
            if is_editing.get() {
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
                            <button class=EDIT_BTN on:click=move |_| {
                                populate_edit_fields();
                                set_is_editing.set(true);
                            }>"Edit"</button>
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
                            <div>
                                <div class="text-xs text-stone-400">"Water Every"</div>
                                <div class="font-medium text-stone-700 dark:text-stone-300">{move || format!("{} days", orchid_signal.get().water_frequency_days)}</div>
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
        <CareScheduleCard orchid_signal=orchid_signal set_orchid_signal=set_orchid_signal />

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
        <div class="flex gap-3 justify-between items-center p-4 mb-4 rounded-xl bg-secondary">
            <div>
                <div class="text-xs tracking-wide text-stone-400">"Watering Status"</div>
                <div class="text-sm font-medium text-stone-700 dark:text-stone-300">
                    {move || {
                        let o = orchid_signal.get();
                        match o.days_until_due() {
                            Some(days) if days < 0 => format!("Overdue by {} days", -days),
                            Some(0) => "Due today".to_string(),
                            Some(1) => "Due tomorrow".to_string(),
                            Some(days) => format!("Due in {} days", days),
                            None => "Never watered".to_string(),
                        }
                    }}
                </div>
            </div>
            <button
                class=BTN_PRIMARY
                disabled=move || is_watering.get()
                on:click=move |_| {
                    set_is_watering.set(true);
                    let orchid_id = orchid_signal.get().id.clone();
                    leptos::task::spawn_local(async move {
                        match crate::server_fns::orchids::mark_watered(orchid_id).await {
                            Ok(updated) => set_orchid_signal.set(updated),
                            Err(e) => log::error!("Failed to mark watered: {}", e),
                        }
                        set_is_watering.set(false);
                    });
                }
            >
                {move || if is_watering.get() { "Watering..." } else { "Water Now" }}
            </button>
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
                                    Err(e) => log::error!("Failed to mark fertilized: {}", e),
                                }
                                set_is_fertilizing.set(false);
                            });
                        }
                    >
                        {move || if is_fertilizing.get() { "..." } else { "\u{2728} Fertilize" }}
                    </button>
                </div>
            </div>

            // Pot info
            <div class="grid grid-cols-2 gap-3 pt-3 mt-3 text-sm border-t border-stone-100 dark:border-stone-700/50">
                <div>
                    <div class=CARE_STAT_LABEL>"\u{1FAB4} Pot Medium"</div>
                    <div class=CARE_STAT_VALUE>
                        {move || orchid_signal.get().pot_medium.unwrap_or_else(|| "Not set".to_string())}
                    </div>
                </div>
                <div>
                    <div class=CARE_STAT_LABEL>"Pot Size"</div>
                    <div class=CARE_STAT_VALUE>
                        {move || orchid_signal.get().pot_size.unwrap_or_else(|| "Not set".to_string())}
                    </div>
                </div>
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
    zones: Vec<GrowingZone>,
    on_save: impl Fn(leptos::ev::SubmitEvent) + 'static + Copy + Send + Sync,
    on_cancel: impl Fn(leptos::ev::MouseEvent) + 'static + Copy + Send + Sync,
) -> impl IntoView {
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
                        <label>"Water Freq (days):"</label>
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
                    <h4 class="mt-0 mb-3 text-xs font-semibold tracking-widest uppercase text-stone-400">"Fertilizer & Pot"</h4>
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
                        <div class="flex-1">
                            <label>"Pot Medium:"</label>
                            <select prop:value=edit_pot_medium on:change=move |ev| set_edit_pot_medium.set(event_target_value(&ev))>
                                <option value="">"Select..."</option>
                                <option value="Bark">"Bark"</option>
                                <option value="Sphagnum Moss">"Sphagnum Moss"</option>
                                <option value="Semi-Hydro (LECA)">"Semi-Hydro (LECA)"</option>
                                <option value="Perlite Mix">"Perlite Mix"</option>
                                <option value="Mounted">"Mounted"</option>
                                <option value="Full Water Culture">"Full Water Culture"</option>
                                <option value="Other">"Other"</option>
                            </select>
                        </div>
                        <div class="flex-1">
                            <label>"Pot Size:"</label>
                            <input type="text" prop:value=edit_pot_size on:input=move |ev| set_edit_pot_size.set(event_target_value(&ev)) placeholder="e.g. 4 inch, 10cm" />
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
