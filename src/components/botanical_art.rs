use leptos::prelude::*;

// ── Full Phalaenopsis Orchid Spray ──────────────────────────────────────
// Arching flower spike with 4 open blooms, buds, and broad leaves at the base.
// Designed as a large background decoration for auth page left panels.

#[component]
pub fn OrchidSpray(
    #[prop(default = "")] class: &'static str,
) -> impl IntoView {
    view! {
        <div class=class inner_html=include_str!("../../public/svg/orchid_spray.svg")></div>
    }
}

// ── Single Orchid Bloom with Short Stem ────────────────────────────────
// A smaller decorative piece — one detailed bloom on a curved stem with a leaf.
// Used as a subtle background accent on the home/main page.

#[component]
pub fn OrchidAccent(
    #[prop(default = "")] class: &'static str,
) -> impl IntoView {
    view! {
        <div class=class inner_html=include_str!("../../public/svg/orchid_accent.svg")></div>
    }
}
