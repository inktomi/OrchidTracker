use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use crate::components::climate_dashboard::ClimateDashboard;
use crate::components::orchid_collection::OrchidCollection;
use crate::components::orchid_detail::OrchidDetail;
use crate::model::ViewMode;
use crate::orchid::Orchid;
use crate::server_fns::public::{
    get_public_orchids, get_public_zones, get_public_climate_readings,
    get_public_hemisphere, get_public_temp_unit,
};

#[component]
pub fn PublicCollectionPage() -> impl IntoView {
    let params = use_params_map();
    let username = Memo::new(move |_| {
        params.get().get("username").unwrap_or_default()
    });

    let orchids_resource = Resource::new(
        move || username.get(),
        |uname| get_public_orchids(uname),
    );

    let zones_resource = Resource::new(
        move || username.get(),
        |uname| get_public_zones(uname),
    );

    let climate_resource = Resource::new(
        move || username.get(),
        |uname| get_public_climate_readings(uname),
    );

    let hemisphere_resource = Resource::new(
        move || username.get(),
        |uname| get_public_hemisphere(uname),
    );

    let temp_unit_resource = Resource::new(
        move || username.get(),
        |uname| get_public_temp_unit(uname),
    );

    let zones_memo = Memo::new(move |_| {
        zones_resource.get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
    });

    let climate_readings = Memo::new(move |_| {
        climate_resource.get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
    });

    let temp_unit = Memo::new(move |_| {
        temp_unit_resource.get()
            .and_then(|r| r.ok())
            .unwrap_or_else(|| "C".to_string())
    });

    let hemisphere = Memo::new(move |_| {
        hemisphere_resource.get()
            .and_then(|r| r.ok())
            .unwrap_or_else(|| "N".to_string())
    });

    let view_mode = Memo::new(move |_| ViewMode::Grid);

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

                    // Strip the server function prefix if present
                    let display_msg = if err_msg.contains("This collection is private") {
                        "This collection is private.".to_string()
                    } else if err_msg.contains("User not found") {
                        "User not found.".to_string()
                    } else {
                        err_msg
                    };

                    return view! {
                        <div class="flex flex-col items-center py-20 px-6 text-center">
                            <div class="mb-4 text-4xl text-stone-300 dark:text-stone-600">"\u{1F512}"</div>
                            <h1 class="mb-2 text-xl font-semibold text-stone-700 dark:text-stone-300">{display_msg}</h1>
                            <p class="text-sm text-stone-400">"The collection you're looking for isn't available."</p>
                        </div>
                    }.into_any();
                }

                let uname = username.get();

                view! {
                    // Header
                    <header class="py-6 px-4 mx-auto text-center max-w-[1200px]">
                        <h1 class="mb-1 text-2xl sm:text-3xl text-stone-800 dark:text-stone-100">
                            {format!("{}\u{2019}s Plant Collection", uname)}
                        </h1>
                        <p class="text-sm text-stone-400">"A shared collection of growing things"</p>
                    </header>

                    <main class="relative z-10 py-2 px-4 mx-auto sm:px-6 max-w-[1200px]">
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
                            orchids_resource=orchids_resource
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
                    </main>

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
                            />
                        }.into_any()
                    })}
                }.into_any()
            }}
        </Suspense>
    }
}
