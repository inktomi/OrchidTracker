use leptos::prelude::*;
use super::event_types::EVENT_TYPES;

#[component]
pub fn EventTypePicker(
    selected: ReadSignal<Option<String>>,
    on_select: impl Fn(Option<String>) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    view! {
        <div class="flex flex-wrap gap-2">
            {EVENT_TYPES.iter().map(|et| {
                let key = et.key;
                let label = et.label;
                let emoji = et.emoji;
                let bg = et.bg_class;
                let color = et.color_class;

                view! {
                    <button
                        type="button"
                        class=move || {
                            let is_selected = selected.get().as_deref() == Some(key);
                            if is_selected {
                                format!("py-1.5 px-3 text-xs font-semibold rounded-full border-2 cursor-pointer transition-all ring-2 ring-offset-1 {} {} border-current", bg, color)
                            } else {
                                format!("py-1.5 px-3 text-xs font-semibold rounded-full border border-transparent cursor-pointer transition-all hover:border-stone-300 {} {}", bg, color)
                            }
                        }
                        on:click=move |_| {
                            let current = selected.get_untracked();
                            if current.as_deref() == Some(key) {
                                on_select(None);
                            } else {
                                on_select(Some(key.to_string()));
                            }
                        }
                    >
                        {emoji} " " {label}
                    </button>
                }
            }).collect::<Vec<_>>()}
        </div>
    }.into_any()
}
