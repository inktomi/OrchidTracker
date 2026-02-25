use crate::components::botanical_art::{OrchidAccent, OrchidSpray};
use crate::components::climate_dashboard::ClimateDashboard;
use crate::components::orchid_collection::OrchidCollection;
use crate::components::orchid_detail::OrchidDetail;
use crate::components::seasonal_calendar::SeasonalCalendar;
use crate::model::ViewMode;
use crate::orchid::Orchid;
use crate::server_fns::auth::get_current_user;
use crate::server_fns::public::{
    get_public_climate_readings, get_public_hemisphere, get_public_orchids, get_public_temp_unit,
    get_public_zones,
};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PublicTab {
    Plants,
    Seasons,
}

const TAB_ACTIVE: &str = "flex gap-2 items-center py-2.5 px-5 text-sm font-semibold border-b-2 cursor-pointer transition-colors text-primary border-primary";
const TAB_INACTIVE: &str = "flex gap-2 items-center py-2.5 px-5 text-sm font-medium border-b-2 border-transparent cursor-pointer transition-colors text-stone-500 hover:text-stone-600";

/// Fixed fullscreen botanical background layer
#[component]
fn PublicBackground() -> impl IntoView {
    view! {
        <div class="overflow-hidden fixed inset-0 z-0 pointer-events-none">
            // Green gradient at top
            <div class="absolute top-0 right-0 left-0 h-64 bg-gradient-to-b to-transparent from-primary/[0.04]"></div>
            // Gold radial glow
            <div class="absolute top-0 right-0 w-96 h-96" style="background: radial-gradient(ellipse at 80% 20%, rgba(182,141,64,0.08), transparent 60%);"></div>
            // Green radial glow
            <div class="absolute bottom-0 left-0 w-96 h-96" style="background: radial-gradient(ellipse at 20% 80%, rgba(45,106,79,0.06), transparent 60%);"></div>
            // Orchid spray — bottom-left, mirrored, low opacity
            <div class="absolute -bottom-8 -left-8 opacity-[0.04] botanical-breathe" style="transform: scaleX(-1);">
                <OrchidSpray class="w-72 h-auto sm:w-80" />
            </div>
            // Orchid accent — top-right, low opacity, delayed animation
            <div class="absolute -top-4 -right-8 opacity-[0.03] botanical-breathe" style="animation-delay: 3s;">
                <OrchidAccent class="w-64 h-auto sm:w-72" />
            </div>
        </div>
    }.into_any()
}

/// Hero section with brand badge, username heading, and plant count
#[component]
fn PublicHero(display_name: String, plant_count: usize) -> impl IntoView {
    view! {
        <header class="relative z-10 py-10 px-4 mx-auto text-center max-w-[1200px] public-hero-in">
            // Brand badge
            <div class="flex gap-2 justify-center items-center mb-5">
                <div class="flex justify-center items-center w-8 h-8 text-sm rounded-lg bg-primary [&>svg]:w-4 [&>svg]:h-4" inner_html=include_str!("../../public/svg/app_logo.svg")></div>
                <span class="text-xs font-semibold tracking-widest uppercase text-primary/80">"Orchid Tracker"</span>
            </div>

            // Main heading
            <h1 class="mb-2 text-3xl sm:text-4xl text-stone-800">
                {format!("{}\u{2019}s ", display_name)}
                <span class="text-accent">"Collection"</span>
            </h1>
            <p class="mb-4 text-sm text-stone-500">"A shared collection of growing things"</p>

            // Plant count pill
            {(plant_count > 0).then(move || view! {
                <div class="inline-flex gap-1.5 items-center py-1 px-3 text-xs font-medium rounded-full bg-primary/[0.08] text-primary">
                    <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 20 20" fill="currentColor">
                        <path d="M10.394 2.08a1 1 0 00-.788 0l-7 3a1 1 0 000 1.84L5.25 8.051a.999.999 0 01.356-.257l4-1.714a1 1 0 11.788 1.838L7.667 9.088l1.94.831a1 1 0 00.787 0l7-3a1 1 0 000-1.838l-7-3zM3.31 9.397L5 10.12v4.102a8.969 8.969 0 00-1.05-.174 1 1 0 01-.89-.89 11.115 11.115 0 01.25-3.762zM9.3 16.573A9.026 9.026 0 007 14.935v-3.957l1.818.78a3 3 0 002.364 0l5.508-2.361a11.026 11.026 0 01.25 3.762 1 1 0 01-.89.89 8.968 8.968 0 00-5.35 2.524 1 1 0 01-1.4 0zM6 18a1 1 0 001-1v-2.065a8.935 8.935 0 00-2-.712V17a1 1 0 001 1z" />
                    </svg>
                    {format!("{} plant{}", plant_count, if plant_count == 1 { "" } else { "s" })}
                </div>
            })}
        </header>
    }.into_any()
}

/// Call-to-action section for unauthenticated visitors
#[component]
fn PublicCTA() -> impl IntoView {
    view! {
        <section class="relative z-10 py-8 px-4 mx-auto max-w-[1200px] public-cta-in">
            <div class="p-6 mx-auto max-w-2xl rounded-2xl border sm:p-8 bg-surface border-stone-200">
                <h2 class="mb-2 text-xl text-center sm:text-2xl text-stone-800">"Start Your Own Collection"</h2>
                <p class="mx-auto mb-5 max-w-md text-sm text-center text-stone-500">
                    "Track your orchids, monitor growing conditions, and get AI-powered species identification — all in one place."
                </p>

                // Feature pills
                <div class="flex flex-wrap gap-2 justify-center mb-6">
                    <span class="py-1 px-3 text-xs font-medium rounded-full bg-primary/[0.08] text-primary">"AI Species ID"</span>
                    <span class="py-1 px-3 text-xs font-medium rounded-full bg-accent/[0.10] text-accent-dark">"Climate Monitoring"</span>
                    <span class="py-1 px-3 text-xs font-medium rounded-full bg-primary/[0.08] text-primary">"Seasonal Calendar"</span>
                    <span class="py-1 px-3 text-xs font-medium rounded-full bg-accent/[0.10] text-accent-dark">"Free & Self-Hosted"</span>
                </div>

                // CTA button
                <div class="text-center">
                    <a
                        href="/register"
                        class="inline-flex gap-2 items-center py-2.5 px-6 text-sm font-semibold text-white rounded-xl transition-all cursor-pointer bg-primary hover:bg-primary-dark"
                    >
                        "Get Started"
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                            <path fill-rule="evenodd" d="M10.293 3.293a1 1 0 011.414 0l6 6a1 1 0 010 1.414l-6 6a1 1 0 01-1.414-1.414L14.586 11H3a1 1 0 110-2h11.586l-4.293-4.293a1 1 0 010-1.414z" clip-rule="evenodd" />
                        </svg>
                    </a>
                </div>
            </div>
        </section>
    }.into_any()
}

#[component]
pub fn PublicCollectionPage() -> impl IntoView {
    let params = use_params_map();
    let username = Memo::new(move |_| params.get().get("username").unwrap_or_default());

    let orchids_resource = Resource::new(move || username.get(), get_public_orchids);

    let zones_resource = Resource::new(move || username.get(), get_public_zones);

    let climate_resource = Resource::new(move || username.get(), get_public_climate_readings);

    let hemisphere_resource = Resource::new(move || username.get(), get_public_hemisphere);

    let temp_unit_resource = Resource::new(move || username.get(), get_public_temp_unit);

    // Auth check for CTA visibility
    let current_user = Resource::new(|| (), |_| get_current_user());

    let zones_memo = Memo::new(move |_| {
        zones_resource
            .get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
    });

    let climate_readings = Memo::new(move |_| {
        climate_resource
            .get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
    });

    let temp_unit = Memo::new(move |_| {
        temp_unit_resource
            .get()
            .and_then(|r| r.ok())
            .unwrap_or_else(|| "C".to_string())
    });

    let hemisphere = Memo::new(move |_| {
        hemisphere_resource
            .get()
            .and_then(|r| r.ok())
            .unwrap_or_else(|| "N".to_string())
    });

    let orchids_memo = Memo::new(move |_| {
        orchids_resource
            .get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
    });

    let view_mode = Memo::new(move |_| ViewMode::Grid);

    // Tab state (local signal, not URL-based)
    let (active_tab, set_active_tab) = signal(PublicTab::Plants);

    // Selected orchid for detail view
    let (selected_orchid, set_selected_orchid) = signal(Option::<Orchid>::None);

    // No-op callbacks for read-only mode
    let noop_string = |_: String| {};
    let noop_orchid = |_: Orchid| {};
    let noop = || {};

    view! {
        <Suspense fallback=move || view! { <p class="p-8 text-center text-stone-500">"Loading..."</p> }>
            {move || {
                let _ = orchids_resource.get();
                let _ = zones_resource.get();
                let _ = climate_resource.get();
                let _ = hemisphere_resource.get();
                let _ = temp_unit_resource.get();

                // Check if any resource returned an error (private collection or user not found)
                let has_error = orchids_resource.get()
                    .map(|r| r.is_err())
                    .unwrap_or(false);

                if has_error {
                    let err_msg = orchids_resource.get()
                        .and_then(|r| r.err())
                        .map(|e| e.to_string())
                        .unwrap_or_else(|| "Something went wrong".to_string());

                    let display_msg = if err_msg.contains("This collection is private") {
                        "This collection is private.".to_string()
                    } else if err_msg.contains("User not found") {
                        "User not found.".to_string()
                    } else {
                        err_msg
                    };

                    return view! {
                        <div class="min-h-screen bg-cream">
                            <PublicBackground />
                            <div class="flex relative z-10 flex-col items-center py-20 px-6 text-center">
                                <div class="flex gap-2 justify-center items-center mb-8">
                                    <div class="flex justify-center items-center w-8 h-8 text-sm rounded-lg bg-primary [&>svg]:w-4 [&>svg]:h-4" inner_html=include_str!("../../public/svg/app_logo.svg")></div>
                                    <span class="text-xs font-semibold tracking-widest uppercase text-primary/80">"Orchid Tracker"</span>
                                </div>
                                <div class="mb-4 text-4xl text-stone-300" aria-hidden="true">"\u{1F512}"</div>
                                <h1 class="mb-2 text-xl font-semibold text-stone-700">{display_msg}</h1>
                                <p class="mb-6 text-sm text-stone-500">"The collection you\u{2019}re looking for isn\u{2019}t available."</p>
                                <a href="/login" class="py-2 px-5 text-sm font-medium text-white rounded-xl transition-colors bg-primary hover:bg-primary-dark">"Sign In"</a>
                            </div>
                        </div>
                    }.into_any();
                }

                let uname = username.get();
                let display_name = capitalize_first(&uname);
                let plant_count = orchids_resource.get()
                    .and_then(|r| r.ok())
                    .map(|v| v.len())
                    .unwrap_or(0);

                // Check if viewer is logged in (for CTA visibility)
                let is_logged_in = move || {
                    current_user.get()
                        .and_then(|r| r.ok())
                        .flatten()
                        .is_some()
                };

                view! {
                    <div class="min-h-screen bg-cream">
                        <PublicBackground />

                        <PublicHero display_name=display_name plant_count=plant_count />

                        <main class="relative z-10 py-2 px-4 mx-auto sm:px-6 max-w-[1200px]">
                            // Tab bar
                            <nav aria-label="Collection navigation" class="flex mb-5 border-b border-stone-200">
                                <button
                                    class=move || if active_tab.get() == PublicTab::Plants { TAB_ACTIVE } else { TAB_INACTIVE }
                                    on:click=move |_| set_active_tab.set(PublicTab::Plants)
                                >
                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                                        <path d="M10.394 2.08a1 1 0 00-.788 0l-7 3a1 1 0 000 1.84L5.25 8.051a.999.999 0 01.356-.257l4-1.714a1 1 0 11.788 1.838L7.667 9.088l1.94.831a1 1 0 00.787 0l7-3a1 1 0 000-1.838l-7-3zM3.31 9.397L5 10.12v4.102a8.969 8.969 0 00-1.05-.174 1 1 0 01-.89-.89 11.115 11.115 0 01.25-3.762zM9.3 16.573A9.026 9.026 0 007 14.935v-3.957l1.818.78a3 3 0 002.364 0l5.508-2.361a11.026 11.026 0 01.25 3.762 1 1 0 01-.89.89 8.968 8.968 0 00-5.35 2.524 1 1 0 01-1.4 0zM6 18a1 1 0 001-1v-2.065a8.935 8.935 0 00-2-.712V17a1 1 0 001 1z" />
                                    </svg>
                                    "Plants"
                                </button>
                                <button
                                    class=move || if active_tab.get() == PublicTab::Seasons { TAB_ACTIVE } else { TAB_INACTIVE }
                                    on:click=move |_| set_active_tab.set(PublicTab::Seasons)
                                >
                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                                        <path fill-rule="evenodd" d="M6 2a1 1 0 00-1 1v1H4a2 2 0 00-2 2v10a2 2 0 002 2h12a2 2 0 002-2V6a2 2 0 00-2-2h-1V3a1 1 0 10-2 0v1H7V3a1 1 0 00-1-1zm0 5a1 1 0 000 2h8a1 1 0 100-2H6z" clip-rule="evenodd" />
                                    </svg>
                                    "Seasons"
                                </button>
                            </nav>

                            // Tab content
                            {move || {
                                match active_tab.get() {
                                    PublicTab::Plants => view! {
                                        <div>
                                            <Suspense fallback=|| ()>
                                                {move || {
                                                    let readings = climate_readings.get();
                                                    let current_zones = zones_memo.get();
                                                    let tu_inner = temp_unit.get();
                                                    view! { <ClimateDashboard
                                                        readings=readings
                                                        zones=current_zones
                                                        unit=temp_unit
                                                        on_show_wizard=|_| {}
                                                        on_zones_changed=|| {}
                                                        temp_unit_str=tu_inner
                                                        read_only=true
                                                    /> }
                                                }}
                                            </Suspense>

                                            <OrchidCollection
                                                orchids=orchids_memo
                                                zones=zones_memo
                                                view_mode=view_mode
                                                on_set_view=|_| {}
                                                on_delete=noop_string
                                                on_select=move |o: Orchid| set_selected_orchid.set(Some(o))
                                                on_update=noop_orchid
                                                on_water=noop_string
                                                on_add=noop
                                                on_scan=noop
                                                read_only=true
                                            />
                                        </div>
                                    }.into_any(),
                                    PublicTab::Seasons => view! {
                                        <div>
                                            <Suspense fallback=|| ()>
                                                {move || {
                                                    let orchids = orchids_resource.get()
                                                        .and_then(|r| r.ok())
                                                        .unwrap_or_default();
                                                    let hemi = hemisphere.get();
                                                    view! { <SeasonalCalendar orchids=orchids hemisphere=hemi /> }
                                                }}
                                            </Suspense>
                                        </div>
                                    }.into_any(),
                                }
                            }}
                        </main>

                        // CTA for unauthenticated visitors
                        {move || (!is_logged_in()).then(|| view! { <PublicCTA /> })}

                        // Detail modal (read-only)
                        {move || selected_orchid.get().map(|orchid| {
                            let current_zones = zones_memo.get();
                            let current_readings = climate_readings.get();
                            let current_hemi = hemisphere.get();
                            view! {
                                <OrchidDetail
                                    orchid=orchid
                                    zones=current_zones
                                    climate_readings=current_readings
                                    hemisphere=current_hemi
                                    on_close=move || set_selected_orchid.set(None)
                                    on_update=noop_orchid
                                    read_only=true
                                    public_username=username.get()
                                />
                            }.into_any()
                        })}
                    </div>
                }.into_any()
            }}
        </Suspense>
    }
}
