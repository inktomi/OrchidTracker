use leptos::prelude::*;
use crate::orchid::LogEntry;
use crate::components::event_types::get_event_info;
use chrono::Local;

const GALLERY_GRID: &str = "grid grid-cols-2 gap-2 sm:grid-cols-3";

/// Photo-only view of an orchid's log entries, displayed as a
/// chronological filmstrip grid with a full-screen lightbox
/// supporting prev/next navigation and side-by-side compare.
#[component]
pub fn PhotoGallery(
    entries: ReadSignal<Vec<LogEntry>>,
) -> impl IntoView {
    // Active lightbox index (None = closed)
    let (lightbox_idx, set_lightbox_idx) = signal(Option::<usize>::None);
    // Compare mode: show current vs. a second photo side-by-side
    let (compare_idx, set_compare_idx) = signal(Option::<usize>::None);

    view! {
        <div>
            {move || {
                let all = entries.get();
                let photos: Vec<(usize, LogEntry)> = all.iter().enumerate()
                    .filter(|(_, e)| e.image_filename.is_some())
                    .map(|(i, e)| (i, e.clone()))
                    .collect();

                if photos.is_empty() {
                    return view! {
                        <div class="py-12 text-center">
                            <div class="mb-2 text-3xl text-stone-300 dark:text-stone-600">"\u{1F4F7}"</div>
                            <p class="text-sm italic text-stone-400">"No photos yet. Add your first growth photo!"</p>
                        </div>
                    }.into_any();
                }

                let photo_count = photos.len();
                view! {
                    <div class="mb-2 text-xs text-stone-400">
                        {format!("{} photo{}", photo_count, if photo_count == 1 { "" } else { "s" })}
                    </div>
                    <div class=GALLERY_GRID>
                        {photos.into_iter().map(|(orig_idx, entry)| {
                            let filename = entry.image_filename.clone().unwrap_or_default();
                            let info = entry.event_type.as_deref().and_then(get_event_info);
                            let badge = info.map(|i| format!("{} {}", i.emoji, i.label));
                            let badge_class = info.map(|i| format!("{} {}", i.bg_class, i.color_class));
                            let ts = entry.timestamp.with_timezone(&Local).format("%b %d").to_string();
                            view! {
                                <div
                                    class="overflow-hidden relative rounded-lg border transition-colors cursor-pointer aspect-square group border-stone-200 dark:border-stone-700 hover:border-primary-light/40"
                                    on:click=move |_| set_lightbox_idx.set(Some(orig_idx))
                                >
                                    <img
                                        src=format!("/images/{}", filename)
                                        class="object-cover w-full h-full transition-transform duration-300 group-hover:scale-105"
                                        alt="Growth photo"
                                        loading="lazy"
                                    />
                                    <div class="absolute inset-x-0 bottom-0 p-2 bg-gradient-to-t to-transparent from-black/60">
                                        <div class="text-xs font-medium text-white/90">{ts}</div>
                                        {badge.map(|b| {
                                            let bc = badge_class.clone().unwrap_or_default();
                                            view! {
                                                <span class=format!("inline-block mt-0.5 py-0.5 px-1.5 text-[10px] font-medium rounded-full {}", bc)>{b}</span>
                                            }
                                        })}
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            }}
        </div>

        // Gallery Lightbox with prev/next + compare
        {move || lightbox_idx.get().map(|idx| {
            let all = entries.get();
            let photo_entries: Vec<(usize, LogEntry)> = all.iter().enumerate()
                .filter(|(_, e)| e.image_filename.is_some())
                .map(|(i, e)| (i, e.clone()))
                .collect();

            // Find position of current idx in photo list
            let current_pos = photo_entries.iter().position(|(i, _)| *i == idx);
            if current_pos.is_none() || photo_entries.is_empty() {
                set_lightbox_idx.set(None);
                return view! { <div></div> }.into_any();
            }
            let pos = current_pos.unwrap();
            let entry = photo_entries[pos].1.clone();
            let can_prev = pos > 0;
            let can_next = pos < photo_entries.len() - 1;
            let prev_idx = if can_prev { Some(photo_entries[pos - 1].0) } else { None };
            let next_idx = if can_next { Some(photo_entries[pos + 1].0) } else { None };

            let filename = entry.image_filename.clone().unwrap_or_default();
            let note = entry.note.clone();
            let info = entry.event_type.as_deref().and_then(get_event_info);
            let event_label = info.map(|i| format!("{} {}", i.emoji, i.label));
            let ts_str = entry.timestamp.with_timezone(&Local).format("%B %d, %Y at %H:%M").to_string();

            // Compare mode rendering
            let compare_entry = compare_idx.get().and_then(|ci| {
                photo_entries.iter().find(|(i, _)| *i == ci).map(|(_, e)| e.clone())
            });
            let is_comparing = compare_entry.is_some();

            view! {
                <GalleryLightbox
                    filename=filename
                    note=note
                    timestamp=ts_str
                    event_label=event_label
                    can_prev=can_prev
                    can_next=can_next
                    is_comparing=is_comparing
                    compare_entry=compare_entry
                    on_close=move || {
                        set_lightbox_idx.set(None);
                        set_compare_idx.set(None);
                    }
                    on_prev=move || {
                        if let Some(pi) = prev_idx {
                            set_lightbox_idx.set(Some(pi));
                            set_compare_idx.set(None);
                        }
                    }
                    on_next=move || {
                        if let Some(ni) = next_idx {
                            set_lightbox_idx.set(Some(ni));
                            set_compare_idx.set(None);
                        }
                    }
                    on_compare=move || {
                        // Toggle compare: use the previous photo
                        if is_comparing {
                            set_compare_idx.set(None);
                        } else if let Some(pi) = prev_idx {
                            set_compare_idx.set(Some(pi));
                        }
                    }
                />
            }.into_any()
        })}
    }.into_any()
}

// ── Gallery Lightbox ─────────────────────────────────────────────────

#[component]
fn GalleryLightbox(
    filename: String,
    note: String,
    timestamp: String,
    event_label: Option<String>,
    can_prev: bool,
    can_next: bool,
    is_comparing: bool,
    compare_entry: Option<LogEntry>,
    on_close: impl Fn() + 'static + Clone + Send + Sync,
    on_prev: impl Fn() + 'static + Clone + Send + Sync,
    on_next: impl Fn() + 'static + Clone + Send + Sync,
    on_compare: impl Fn() + 'static + Clone + Send + Sync,
) -> impl IntoView {
    let on_close2 = on_close.clone();

    view! {
        <div
            class="flex fixed inset-0 flex-col justify-center items-center cursor-pointer z-[2000] bg-black/90 animate-fade-in"
            on:click=move |_| on_close()
        >
            // Top bar: close + compare toggle
            <div
                class="flex absolute top-0 right-0 left-0 gap-3 justify-between items-center py-3 px-4 z-[2010]"
                on:click=move |ev: leptos::ev::MouseEvent| ev.stop_propagation()
            >
                <div class="flex gap-2 items-center">
                    {can_prev.then(|| {
                        let op = on_compare.clone();
                        view! {
                            <button
                                class="py-1.5 px-3 text-xs font-medium rounded-lg border transition-colors cursor-pointer hover:text-white text-white/80 border-white/20 bg-white/10 hover:bg-white/20"
                                on:click=move |_| op()
                            >
                                {if is_comparing { "Exit Compare" } else { "Compare" }}
                            </button>
                        }
                    })}
                </div>
                <button
                    class="text-2xl bg-transparent border-none cursor-pointer hover:text-white text-white/70"
                    on:click=move |_| on_close2()
                >"\u{00D7}"</button>
            </div>

            // Navigation arrows
            {can_prev.then(|| {
                let op = on_prev.clone();
                view! {
                    <button
                        class="flex absolute left-2 top-1/2 z-20 justify-center items-center w-10 h-10 text-xl bg-transparent rounded-full border-none transition-colors -translate-y-1/2 cursor-pointer hover:text-white text-white/70 hover:bg-white/10"
                        on:click=move |ev: leptos::ev::MouseEvent| { ev.stop_propagation(); op(); }
                    >"\u{2039}"</button>
                }
            })}
            {can_next.then(|| {
                let on = on_next.clone();
                view! {
                    <button
                        class="flex absolute right-2 top-1/2 z-20 justify-center items-center w-10 h-10 text-xl bg-transparent rounded-full border-none transition-colors -translate-y-1/2 cursor-pointer hover:text-white text-white/70 hover:bg-white/10"
                        on:click=move |ev: leptos::ev::MouseEvent| { ev.stop_propagation(); on(); }
                    >"\u{203A}"</button>
                }
            })}

            // Image area
            <div
                class={if is_comparing {
                    "flex gap-2 justify-center items-center w-full max-w-[95vw] max-h-[75vh] px-12"
                } else {
                    "flex justify-center items-center px-12"
                }}
                on:click=move |ev: leptos::ev::MouseEvent| ev.stop_propagation()
            >
                {if is_comparing {
                    // Side-by-side compare
                    let compare = compare_entry.clone();
                    let compare_fname = compare.as_ref()
                        .and_then(|e| e.image_filename.clone())
                        .unwrap_or_default();
                    let compare_ts = compare.as_ref()
                        .map(|e| e.timestamp.with_timezone(&Local).format("%b %d, %Y").to_string())
                        .unwrap_or_default();
                    let current_ts = timestamp.clone();
                    let fname = filename.clone();
                    view! {
                        <div class="flex gap-3 w-full">
                            <div class="flex-1 text-center">
                                <img
                                    src=format!("/images/{}", compare_fname)
                                    class="object-contain mx-auto rounded-lg max-h-[65vh]"
                                    alt="Earlier photo"
                                />
                                <div class="mt-2 text-xs text-stone-400">{compare_ts}" (earlier)"</div>
                            </div>
                            <div class="self-stretch w-px bg-white/20"></div>
                            <div class="flex-1 text-center">
                                <img
                                    src=format!("/images/{}", fname)
                                    class="object-contain mx-auto rounded-lg max-h-[65vh]"
                                    alt="Current photo"
                                />
                                <div class="mt-2 text-xs text-stone-400">{current_ts}" (current)"</div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    let fname = filename.clone();
                    view! {
                        <img
                            src=format!("/images/{}", fname)
                            class="object-contain rounded-lg max-w-[90vw] max-h-[75vh]"
                            alt="Full size photo"
                        />
                    }.into_any()
                }}
            </div>

            // Info bar at bottom
            <div
                class="absolute inset-x-0 bottom-0 py-3 px-4 text-center"
                on:click=move |ev: leptos::ev::MouseEvent| ev.stop_propagation()
            >
                <div class="mb-1 text-xs text-stone-400">{timestamp.clone()}</div>
                {event_label.map(|label| {
                    view! { <span class="py-0.5 px-2 text-xs font-medium rounded-full text-white/70 bg-white/10">{label}</span> }
                })}
                {(!note.is_empty()).then(|| {
                    view! { <p class="mt-1 text-sm text-white/70">{note.clone()}</p> }
                })}
            </div>
        </div>
    }.into_any()
}
