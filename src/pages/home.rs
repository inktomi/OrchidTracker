use leptos::prelude::*;
use crate::components::add_orchid_form::AddOrchidForm;
use crate::components::app_header::AppHeader;
use crate::components::climate_dashboard::ClimateDashboard;
use crate::components::orchid_collection::OrchidCollection;
use crate::components::orchid_detail::OrchidDetail;
use crate::components::scanner::ScannerModal;
use crate::components::settings::SettingsModal;
use crate::model::{Model, Msg};
use crate::orchid::Orchid;
use crate::server_fns::auth::get_current_user;
use crate::server_fns::orchids::{get_orchids, create_orchid, update_orchid, delete_orchid};
use crate::update::dispatch;

#[component]
pub fn HomePage() -> impl IntoView {
    // Check auth — redirect to login if not authenticated
    let user = Resource::new(|| (), |_| get_current_user());

    // Load orchids from server
    let orchids_resource = Resource::new(|| (), |_| get_orchids());

    // TEA model + dispatch
    let (model, set_model) = signal(Model::default());
    let send = move |msg: Msg| dispatch(set_model, model, msg);

    // Derived memos for fine-grained reactivity
    let view_mode = Memo::new(move |_| model.get().view_mode);
    let selected_orchid = Memo::new(move |_| model.get().selected_orchid.clone());
    let show_settings = Memo::new(move |_| model.get().show_settings);
    let show_scanner = Memo::new(move |_| model.get().show_scanner);
    let show_add_modal = Memo::new(move |_| model.get().show_add_modal);
    let prefill_data = Memo::new(move |_| model.get().prefill_data.clone());
    let temp_unit = Memo::new(move |_| model.get().temp_unit.clone());
    let dark_mode = Memo::new(move |_| model.get().dark_mode);

    // Static climate data (included at compile time)
    let climate_data: Vec<crate::app::ClimateData> =
        serde_json::from_str(include_str!("../data/climate.json")).unwrap_or_default();
    let climate_data = StoredValue::new(climate_data);

    // Orchid operations via server functions (async I/O — not TEA state)
    let on_add = move |orchid: Orchid| {
        leptos::task::spawn_local(async move {
            let _ = create_orchid(
                orchid.name,
                orchid.species,
                orchid.water_frequency_days,
                orchid.light_requirement.to_string(),
                orchid.notes,
                orchid.placement.to_string(),
                orchid.light_lux,
                orchid.temperature_range,
                orchid.conservation_status,
            ).await;
            orchids_resource.refetch();
        });
    };

    let on_update = move |orchid: Orchid| {
        leptos::task::spawn_local(async move {
            let _ = update_orchid(orchid.clone()).await;
            orchids_resource.refetch();
        });
    };

    let on_delete = move |id: String| {
        #[cfg(feature = "hydrate")]
        {
            if let Some(window) = web_sys::window() {
                if !window.confirm_with_message("Are you sure you want to delete this orchid?").unwrap_or(false) {
                    return;
                }
            }
        }
        leptos::task::spawn_local(async move {
            let _ = delete_orchid(id).await;
            orchids_resource.refetch();
        });
    };

    view! {
        <Suspense fallback=move || view! { <p class="p-8 text-center text-stone-500">"Loading..."</p> }>
            {move || {
                user.get().map(|result| match result {
                    Ok(Some(_user_info)) => view! { <div></div> }.into_any(),
                    _ => {
                        #[cfg(feature = "ssr")]
                        leptos_axum::redirect("/login");
                        #[cfg(feature = "hydrate")]
                        {
                            if let Some(window) = web_sys::window() {
                                let _ = window.location().set_href("/login");
                            }
                        }
                        view! { <div></div> }.into_any()
                    }
                })
            }}
        </Suspense>

        <AppHeader
            dark_mode=dark_mode
            on_toggle_dark=move || send(Msg::ToggleDarkMode)
            on_add=move || send(Msg::ShowAddModal(true))
            on_scan=move || send(Msg::ShowScanner(true))
            on_settings=move || send(Msg::ShowSettings(true))
        />

        <main class="py-6 px-4 mx-auto sm:px-6 max-w-[1200px]">
            <ClimateDashboard data=climate_data unit=temp_unit />

            <OrchidCollection
                orchids_resource=orchids_resource
                view_mode=view_mode
                on_set_view=move |mode| send(Msg::SetViewMode(mode))
                on_delete=on_delete
                on_select=move |o: Orchid| send(Msg::SelectOrchid(Some(o)))
                on_update=on_update
            />

            {move || show_add_modal.get().then(|| {
                view! {
                    <AddOrchidForm
                        on_add=on_add
                        on_close=move || send(Msg::ShowAddModal(false))
                        prefill_data=prefill_data
                    />
                }
            })}

            {move || selected_orchid.get().map(|orchid| {
                view! {
                    <OrchidDetail
                        orchid=orchid
                        on_close=move || send(Msg::SelectOrchid(None))
                        on_update=on_update
                    />
                }
            })}

            {move || show_settings.get().then(|| {
                view! {
                    <SettingsModal on_close=move |temp_unit: String| send(Msg::SettingsClosed { temp_unit }) />
                }
            })}

            {move || show_scanner.get().then(|| {
                let orchids = orchids_resource.get().and_then(|r| r.ok()).unwrap_or_default();
                view! {
                    <ScannerModal
                        on_close=move || send(Msg::ShowScanner(false))
                        on_add_to_collection=move |result| send(Msg::HandleScanResult(result))
                        existing_orchids=orchids
                        climate_data=climate_data.get_value()
                    />
                }
            })}
        </main>
    }
}
