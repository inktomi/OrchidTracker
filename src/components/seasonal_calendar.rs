use leptos::prelude::*;
use crate::orchid::{Orchid, Hemisphere, month_in_range};
use chrono::Datelike;

#[component]
pub fn SeasonalCalendar(
    orchids: Vec<Orchid>,
    hemisphere: String,
) -> impl IntoView {
    let hemi = Hemisphere::from_code(&hemisphere);
    let seasonal_orchids: Vec<Orchid> = orchids.into_iter()
        .filter(|o| o.has_seasonal_data())
        .collect();

    if seasonal_orchids.is_empty() {
        return view! { <div></div> }.into_any();
    }

    let now_month = chrono::Utc::now().month();

    // Count orchids entering rest next month
    let next_month = if now_month == 12 { 1 } else { now_month + 1 };
    let entering_rest = seasonal_orchids.iter().filter(|o| {
        o.rest_start_month.map(|s| hemi.adjust_month(s) == next_month).unwrap_or(false)
    }).count();
    let entering_bloom = seasonal_orchids.iter().filter(|o| {
        o.bloom_start_month.map(|s| hemi.adjust_month(s) == next_month).unwrap_or(false)
    }).count();

    let hemi_for_rows = hemi.clone();

    let rows = seasonal_orchids.iter().map(|orchid| {
        let name = orchid.name.clone();
        let cells = (1..=12u32).map(|m| {
            let in_rest = orchid.rest_start_month.zip(orchid.rest_end_month)
                .map(|(s, e)| month_in_range(m, hemi_for_rows.adjust_month(s), hemi_for_rows.adjust_month(e)))
                .unwrap_or(false);
            let in_bloom = orchid.bloom_start_month.zip(orchid.bloom_end_month)
                .map(|(s, e)| month_in_range(m, hemi_for_rows.adjust_month(s), hemi_for_rows.adjust_month(e)))
                .unwrap_or(false);
            let is_current = m == now_month;

            let bg = if in_bloom {
                "bg-pink-200 dark:bg-pink-800/40"
            } else if in_rest {
                "bg-blue-200 dark:bg-blue-800/40"
            } else {
                "bg-emerald-50 dark:bg-emerald-900/20"
            };

            let border = if is_current { " ring-1 ring-primary" } else { "" };
            let class = format!("h-5 rounded-sm {}{}", bg, border);
            view! { <div class=class></div> }
        }).collect::<Vec<_>>();

        view! {
            <div class="grid gap-0.5 items-center" style="grid-template-columns: 120px repeat(12, 1fr)">
                <div class="pr-2 text-xs font-medium truncate text-stone-600 dark:text-stone-400">{name}</div>
                {cells}
            </div>
        }
    }).collect::<Vec<_>>();

    // Month headers
    let month_headers = (1..=12u32).map(|m| {
        let is_current = m == now_month;
        let class = if is_current {
            "text-center font-semibold text-primary text-[10px]"
        } else {
            "text-center text-stone-400 text-[10px]"
        };
        view! { <div class=class>{Orchid::month_name(m)}</div> }
    }).collect::<Vec<_>>();

    view! {
        <div class="p-4 mb-4 rounded-xl border border-stone-200 dark:border-stone-700">
            <div class="flex gap-2 justify-between items-center mb-3">
                <h3 class="m-0 text-sm font-semibold tracking-wide text-stone-500 dark:text-stone-400">"Seasonal Calendar"</h3>
                <div class="flex gap-3 text-xs text-stone-400">
                    <span class="flex gap-1 items-center"><span class="inline-block w-2.5 h-2.5 bg-blue-200 rounded-sm dark:bg-blue-800/40"></span>"Rest"</span>
                    <span class="flex gap-1 items-center"><span class="inline-block w-2.5 h-2.5 bg-pink-200 rounded-sm dark:bg-pink-800/40"></span>"Bloom"</span>
                    <span class="flex gap-1 items-center"><span class="inline-block w-2.5 h-2.5 bg-emerald-50 rounded-sm dark:bg-emerald-900/20"></span>"Active"</span>
                </div>
            </div>

            // Month header row
            <div class="grid gap-0.5 mb-1" style="grid-template-columns: 120px repeat(12, 1fr)">
                <div></div>
                {month_headers}
            </div>

            // Orchid rows
            <div class="flex flex-col gap-0.5">
                {rows}
            </div>

            // Summary
            {(entering_rest > 0 || entering_bloom > 0).then(|| {
                let mut parts = Vec::new();
                if entering_rest > 0 {
                    parts.push(format!("{} entering rest", entering_rest));
                }
                if entering_bloom > 0 {
                    parts.push(format!("{} entering bloom", entering_bloom));
                }
                let summary = format!("Next month: {}", parts.join(", "));
                view! {
                    <div class="pt-2 mt-2 text-xs border-t text-stone-400 border-stone-100 dark:border-stone-700/50">
                        {summary}
                    </div>
                }
            })}
        </div>
    }.into_any()
}
