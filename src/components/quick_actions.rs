use leptos::prelude::*;
use std::collections::HashMap;
use crate::orchid::{Orchid, LogEntry};
use crate::components::event_types::quick_action_types;

#[derive(Clone, Copy, PartialEq)]
enum BtnState {
    Idle,
    Loading,
    Done,
}

#[component]
pub fn QuickActions(
    orchid_signal: ReadSignal<Orchid>,
    set_orchid_signal: WriteSignal<Orchid>,
    set_log_entries: WriteSignal<Vec<LogEntry>>,
    set_show_first_bloom: WriteSignal<bool>,
) -> impl IntoView {
    let btn_states = RwSignal::new(HashMap::<&'static str, BtnState>::new());

    let buttons = quick_action_types().map(|et| {
        let key = et.key;
        let emoji = et.emoji;
        let label = et.label;
        let bg = et.bg_class;
        let color = et.color_class;

        let state = Memo::new(move |_| {
            btn_states.with(|m| m.get(key).copied().unwrap_or(BtnState::Idle))
        });

        let on_click = move |_: leptos::ev::MouseEvent| {
            if state.get() != BtnState::Idle {
                return;
            }
            btn_states.update(|m| { m.insert(key, BtnState::Loading); });
            let orchid_id = orchid_signal.get().id.clone();
            let event_key = key.to_string();

            leptos::task::spawn_local(async move {
                match crate::server_fns::orchids::add_log_entry(
                    orchid_id,
                    String::new(),
                    None,
                    Some(event_key),
                ).await {
                    Ok(response) => {
                        if response.is_first_bloom {
                            set_show_first_bloom.set(true);
                        }
                        let now = chrono::Utc::now();
                        match key {
                            "Watered" => set_orchid_signal.update(|o| o.last_watered_at = Some(now)),
                            "Fertilized" => set_orchid_signal.update(|o| o.last_fertilized_at = Some(now)),
                            "Repotted" => set_orchid_signal.update(|o| o.last_repotted_at = Some(now)),
                            _ => {}
                        }
                        set_log_entries.update(|entries| entries.insert(0, response.entry));
                        btn_states.update(|m| { m.insert(key, BtnState::Done); });

                        // Reset to idle after 1.5s
                        #[cfg(feature = "hydrate")]
                        {
                            gloo_timers::future::TimeoutFuture::new(1_500).await;
                            btn_states.update(|m| { m.insert(key, BtnState::Idle); });
                        }
                        #[cfg(not(feature = "hydrate"))]
                        {
                            btn_states.update(|m| { m.insert(key, BtnState::Idle); });
                        }
                    }
                    Err(e) => {
                        tracing::error!("Quick action '{}' failed: {}", key, e);
                        btn_states.update(|m| { m.insert(key, BtnState::Idle); });
                    }
                }
            });
        };

        let btn_class = move || {
            let s = state.get();
            let base = format!(
                "inline-flex gap-1.5 items-center py-1.5 px-3 text-sm font-medium rounded-full border-none cursor-pointer transition-all {} {}",
                bg, color
            );
            match s {
                BtnState::Loading => format!("{} animate-pulse opacity-70", base),
                BtnState::Done => format!("{} ring-2 ring-emerald-400 ring-offset-1", base),
                BtnState::Idle => format!("{} hover:scale-105 active:scale-95", base),
            }
        };

        view! {
            <button
                class=btn_class
                on:click=on_click
                disabled=move || state.get() == BtnState::Loading
            >
                <span>{emoji}</span>
                <span>{move || match state.get() {
                    BtnState::Done => "\u{2713}",
                    _ => label,
                }}</span>
            </button>
        }
    }).collect::<Vec<_>>();

    view! {
        <div class="mb-4">
            <h4 class="mt-0 mb-2 text-xs font-semibold tracking-widest uppercase text-stone-500 dark:text-stone-400">"Quick Log"</h4>
            <div class="flex flex-wrap gap-2">
                {buttons}
            </div>
        </div>
    }.into_any()
}
