use leptos::prelude::*;
use crate::orchid::Orchid;
use crate::estimation::{recommend_potting_setup, VPD_BASELINE};
use crate::watering::ClimateSnapshot;

#[component]
pub fn SuitabilityCard(
    orchid_signal: ReadSignal<Orchid>,
    #[prop(default = None)] climate_snapshot: Option<ClimateSnapshot>,
) -> impl IntoView {
    view! {
        {move || {
            let orchid = orchid_signal.get();
            let home_vpd = climate_snapshot.as_ref().map(|s| s.avg_vpd_kpa).unwrap_or(VPD_BASELINE);
            // Without a proper native VPD lookup table yet, we default to 0.8 for tropicals, 1.2 for others based on light.
            let native_vpd = match orchid.light_requirement {
                crate::orchid::LightRequirement::Low => 0.6,
                crate::orchid::LightRequirement::Medium => 0.9,
                crate::orchid::LightRequirement::High => 1.3,
            };

            let recommendation = recommend_potting_setup(native_vpd, home_vpd);

            view! {
                <div class="p-4 mb-4 rounded-xl border border-stone-200 dark:border-stone-700 bg-stone-50 dark:bg-stone-800/50">
                    <h3 class="flex gap-2 items-center mt-0 mb-3 text-sm font-semibold tracking-wide text-stone-500 dark:text-stone-400">
                        <span class="text-primary dark:text-primary-light">"\u{1F9EA}"</span>
                        "Scientific Suitability"
                    </h3>
                    <p class="mb-3 text-sm leading-relaxed text-stone-600 dark:text-stone-300">
                        {recommendation.scientific_reasoning}
                    </p>
                    <div class="grid grid-cols-2 gap-3 text-sm border-t pt-3 border-stone-200/60 dark:border-stone-700/60">
                        <div>
                            <div class="text-xs tracking-wide text-stone-400">"Recommended Medium"</div>
                            <div class="font-medium text-stone-700 dark:text-stone-300">{recommendation.suggested_medium.to_string()}</div>
                        </div>
                        <div>
                            <div class="text-xs tracking-wide text-stone-400">"Recommended Pot"</div>
                            <div class="font-medium text-stone-700 dark:text-stone-300">{recommendation.suggested_pot_type.to_string()}</div>
                        </div>
                    </div>
                </div>
            }
        }}
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use crate::test_helpers::{test_climate_snapshot_hot_dry, test_orchid};

    #[test]
    fn test_suitability_card_renders_recommendations() {
        let owner = leptos::reactive::owner::Owner::new();
        owner.with(|| {
            let (orchid_signal, _) = signal(test_orchid());
            // Using hot/dry snapshot (high VPD) means it will recommend Sphagnum & Solid pot
            let snap = test_climate_snapshot_hot_dry();
            
            let html = view! {
                <SuitabilityCard
                    orchid_signal=orchid_signal
                    climate_snapshot=Some(snap)
                />
            }.to_html();
            
            assert!(html.contains("Scientific Suitability"));
            assert!(html.contains("Sphagnum Moss"));
            assert!(html.contains("Solid"));
            assert!(html.contains("significantly drier"));
        });
    }
}
