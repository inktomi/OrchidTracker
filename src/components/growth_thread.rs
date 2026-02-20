use leptos::prelude::*;
use crate::orchid::LogEntry;
use crate::components::event_types::get_event_info;
use chrono::{Datelike, Local};

const THREAD_LINE: &str = "absolute left-[18px] top-0 bottom-0 w-0.5 bg-primary-light/30";

#[component]
pub fn GrowthThread(
    entries: ReadSignal<Vec<LogEntry>>,
    #[prop(optional)] orchid_id: Option<String>,
) -> impl IntoView {
    let orchid_id = StoredValue::new(orchid_id.unwrap_or_default());
    view! {
        <div class="relative">
            // Thread vine line
            <div class=THREAD_LINE></div>

            {move || {
                let all_entries = entries.get();
                if all_entries.is_empty() {
                    return view! {
                        <div class="py-8 text-sm italic text-center text-stone-400">
                            "No entries yet. Add your first growth note!"
                        </div>
                    }.into_any();
                }

                // Group entries by month
                let mut groups: Vec<(String, Vec<LogEntry>)> = Vec::new();
                for entry in &all_entries {
                    let local = entry.timestamp.with_timezone(&Local);
                    let month_key = format!("{} {}", month_name(local.month()), local.year());
                    if let Some(last) = groups.last_mut() {
                        if last.0 == month_key {
                            last.1.push(entry.clone());
                            continue;
                        }
                    }
                    groups.push((month_key, vec![entry.clone()]));
                }

                let oid = orchid_id.get_value();
                view! {
                    <div>
                        {groups.into_iter().map(move |group| {
                            let (month, month_entries) = group;
                            let oid = oid.clone();
                            view! {
                                <MonthSection month=month entries=month_entries orchid_id=oid />
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            }}
        </div>
    }.into_any()
}

#[component]
fn MonthSection(
    month: String,
    entries: Vec<LogEntry>,
    orchid_id: String,
) -> impl IntoView {
    view! {
        <div class="mb-2">
            // Month divider
            <div class="flex relative gap-3 items-center py-3 pl-10">
                // Tendril decoration on the thread
                <div class="absolute top-1/2 w-3 h-3 -translate-y-1/2 left-[13px]">
                    <svg viewBox="0 0 12 12" class="w-3 h-3 text-primary-light/40">
                        <path d="M6 0 C6 4, 10 6, 6 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                    </svg>
                </div>
                <span class="text-sm font-display text-stone-400 dark:text-stone-500">{month}</span>
                <div class="flex-1 h-px bg-stone-200 dark:bg-stone-700"></div>
            </div>

            // Entries for this month
            {entries.into_iter().map({
                let orchid_id = orchid_id.clone();
                move |entry| {
                    let event_type = entry.event_type.clone();
                    let is_watering = event_type.as_deref() == Some("Watered");
                    let is_milestone = matches!(event_type.as_deref(), Some("Flowering" | "Purchased" | "Repotted"));
                    let has_photo = entry.image_filename.is_some();

                    if has_photo {
                        view! { <PhotoNode entry=entry /> }.into_any()
                    } else if is_watering {
                        view! { <WateringNode entry=entry /> }.into_any()
                    } else if is_milestone {
                        let oid = orchid_id.clone();
                        view! { <MilestoneNode entry=entry orchid_id=oid /> }.into_any()
                    } else {
                        view! { <TextNode entry=entry /> }.into_any()
                    }
                }
            }).collect::<Vec<_>>()}
        </div>
    }.into_any()
}

#[component]
fn PhotoNode(entry: LogEntry) -> impl IntoView {
    let info = entry.event_type.as_deref().and_then(get_event_info);
    let badge_class = info.map(|i| format!("{} {}", i.bg_class, i.color_class)).unwrap_or_default();
    let badge_text = info.map(|i| format!("{} {}", i.emoji, i.label));
    let filename = entry.image_filename.clone().unwrap_or_default();
    let (show_lightbox, set_show_lightbox) = signal(false);
    let note = entry.note.clone();
    let note_for_lightbox = entry.note.clone();
    let filename_for_lightbox = filename.clone();
    let timestamp = entry.timestamp;

    view! {
        <div class="relative pb-4 pl-10">
            // Dot on thread
            <div class="absolute top-2 z-10 w-3 h-3 rounded-full border-2 left-[14px] bg-primary-light border-surface"></div>

            // Timestamp
            <div class="mb-1 text-xs text-stone-400">
                {timestamp.with_timezone(&Local).format("%b %d, %H:%M").to_string()}
            </div>

            // Photo
            <div class="overflow-hidden relative mb-2 rounded-xl border cursor-pointer border-stone-200 dark:border-stone-700"
                on:click=move |_| set_show_lightbox.set(true)
            >
                <img
                    src=format!("/images/{}", filename)
                    class="block object-cover w-full max-h-[400px]"
                    alt="Growth photo"
                    loading="lazy"
                />
                {badge_text.clone().map(|text| {
                    let bc = badge_class.clone();
                    view! {
                        <span class=format!("absolute top-2 right-2 py-1 px-2 text-xs font-semibold rounded-full {}", bc)>{text}</span>
                    }
                })}
            </div>

            // Note
            {(!note.is_empty()).then(|| {
                view! { <p class="text-sm text-stone-700 dark:text-stone-300">{note.clone()}</p> }
            })}
        </div>

        // Lightbox
        {move || show_lightbox.get().then(|| {
            let fname = filename_for_lightbox.clone();
            let n = note_for_lightbox.clone();
            view! {
                <PhotoLightbox
                    filename=fname
                    note=n
                    timestamp=timestamp
                    on_close=move || set_show_lightbox.set(false)
                />
            }
        })}
    }.into_any()
}

#[component]
fn TextNode(entry: LogEntry) -> impl IntoView {
    let info = entry.event_type.as_deref().and_then(get_event_info);
    let dot_color = info.map(|i| i.color_class).unwrap_or("text-stone-400");
    let badge = info.map(|i| format!("{} {}", i.emoji, i.label));
    let badge_classes = info.map(|i| format!("{} {}", i.bg_class, i.color_class));

    view! {
        <div class="relative pb-3 pl-10">
            // Colored dot
            <div class=format!("absolute left-[15px] top-[0.4rem] w-2.5 h-2.5 rounded-full border-2 border-surface z-10 bg-current {}", dot_color)></div>

            <div class="flex flex-wrap gap-2 items-baseline">
                <span class="text-xs text-stone-400">
                    {entry.timestamp.with_timezone(&Local).format("%b %d, %H:%M").to_string()}
                </span>
                {badge.map(|b| {
                    let bc = badge_classes.clone().unwrap_or_default();
                    view! {
                        <span class=format!("py-0.5 px-2 text-xs font-medium rounded-full {}", bc)>{b}</span>
                    }
                })}
            </div>
            {(!entry.note.is_empty()).then(|| {
                view! { <p class="mt-0.5 text-sm text-stone-700 dark:text-stone-300">{entry.note.clone()}</p> }
            })}
        </div>
    }.into_any()
}

#[component]
fn WateringNode(entry: LogEntry) -> impl IntoView {
    view! {
        <div class="relative pb-1.5 pl-10">
            // Small droplet dot
            <div class="absolute z-10 w-2 h-2 rounded-full left-[16px] top-[0.35rem] bg-sky-400/60"></div>
            <div class="flex gap-2 items-baseline">
                <span class="text-xs text-stone-400">
                    {entry.timestamp.with_timezone(&Local).format("%b %d").to_string()}
                </span>
                <span class="text-xs text-sky-500 dark:text-sky-400">
                    "\u{1F4A7} Watered"
                </span>
            </div>
        </div>
    }.into_any()
}

#[component]
fn MilestoneNode(entry: LogEntry, orchid_id: String) -> impl IntoView {
    let info = entry.event_type.as_deref().and_then(get_event_info);
    let dot_color = info.map(|i| i.color_class).unwrap_or("text-primary-light");
    let bg_color = info.map(|i| i.bg_class).unwrap_or("bg-primary-light/10");
    let label = info.map(|i| format!("{} {}", i.emoji, i.label)).unwrap_or_default();
    let is_flowering = entry.event_type.as_deref() == Some("Flowering");

    // Care recap state (lazy-loaded)
    let (show_recap, set_show_recap) = signal(false);
    let (recap_text, set_recap_text) = signal(Option::<String>::None);
    let (recap_loading, set_recap_loading) = signal(false);
    let event_type_for_recap = entry.event_type.clone().unwrap_or_default();

    let load_recap = move |_| {
        // Toggle visibility
        if show_recap.get_untracked() {
            set_show_recap.set(false);
            return;
        }
        set_show_recap.set(true);

        // Only fetch if not already cached
        if recap_text.get_untracked().is_some() {
            return;
        }

        set_recap_loading.set(true);
        let oid = orchid_id.clone();
        let et = event_type_for_recap.clone();
        leptos::task::spawn_local(async move {
            match crate::server_fns::scanner::generate_care_recap(oid, et).await {
                Ok(text) => set_recap_text.set(Some(text)),
                Err(e) => set_recap_text.set(Some(format!("Could not generate recap: {}", e))),
            }
            set_recap_loading.set(false);
        });
    };

    view! {
        <div class="relative pb-4 pl-10">
            // Larger milestone dot with glow
            <div class=format!(
                "absolute left-[12px] top-1.5 w-4 h-4 rounded-full border-2 border-surface z-10 bg-current {}{}",
                dot_color,
                if is_flowering { " shadow-[0_0_8px_rgba(236,72,153,0.4)]" } else { "" }
            )></div>

            // Milestone card
            <div class=format!("rounded-xl p-3 {}", bg_color)>
                <div class="flex gap-2 items-baseline mb-1">
                    <span class=format!("text-sm font-semibold {}", dot_color)>{label}</span>
                    <span class="text-xs text-stone-400">
                        {entry.timestamp.with_timezone(&Local).format("%b %d, %Y").to_string()}
                    </span>
                </div>
                {(!entry.note.is_empty()).then(|| {
                    view! { <p class="text-sm text-stone-700 dark:text-stone-300">{entry.note.clone()}</p> }
                })}

                // Care recap expander
                <button
                    type="button"
                    class="mt-2 text-xs font-medium bg-transparent border-none transition-colors cursor-pointer text-stone-400 dark:hover:text-stone-300 hover:text-stone-600"
                    on:click=load_recap
                >
                    {move || if show_recap.get() { "\u{25BC} Hide insight" } else { "\u{2728} Why did this happen?" }}
                </button>
                {move || show_recap.get().then(|| {
                    view! {
                        <div class="p-2.5 mt-2 rounded-lg border bg-accent/5 border-accent/10">
                            {move || {
                                if recap_loading.get() {
                                    view! { <p class="text-xs italic text-stone-400">"Analyzing care history..."</p> }.into_any()
                                } else if let Some(text) = recap_text.get() {
                                    view! {
                                        <p class="text-xs italic text-stone-600 dark:text-stone-400">{text}</p>
                                    }.into_any()
                                } else {
                                    view! { <p class="text-xs text-stone-400">"Loading..."</p> }.into_any()
                                }
                            }}
                        </div>
                    }
                })}
            </div>
        </div>
    }.into_any()
}

#[component]
fn PhotoLightbox(
    filename: String,
    note: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    on_close: impl Fn() + 'static + Clone + Send + Sync,
) -> impl IntoView {
    let on_close2 = on_close.clone();
    view! {
        <div
            class="flex fixed inset-0 flex-col justify-center items-center cursor-pointer z-[2000] bg-black/90 animate-fade-in"
            on:click=move |_| on_close()
        >
            <img
                src=format!("/images/{}", filename)
                class="object-contain rounded-lg max-w-[95vw] max-h-[80vh]"
                alt="Full size photo"
                on:click=move |ev: leptos::ev::MouseEvent| ev.stop_propagation()
            />
            <div class="mt-4 max-w-lg text-center" on:click=move |ev: leptos::ev::MouseEvent| ev.stop_propagation()>
                <div class="mb-1 text-xs text-stone-400">
                    {timestamp.with_timezone(&Local).format("%B %d, %Y at %H:%M").to_string()}
                </div>
                {(!note.is_empty()).then(|| {
                    view! { <p class="text-sm text-white/80">{note.clone()}</p> }
                })}
            </div>
            <button
                class="absolute top-4 right-4 text-2xl bg-transparent border-none cursor-pointer hover:text-white text-white/70"
                on:click=move |_| on_close2()
            >"\u{00D7}"</button>
        </div>
    }.into_any()
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January", 2 => "February", 3 => "March",
        4 => "April", 5 => "May", 6 => "June",
        7 => "July", 8 => "August", 9 => "September",
        10 => "October", 11 => "November", 12 => "December",
        _ => "Unknown",
    }
}
