use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::pages::home::HomePage;
use crate::pages::login::LoginPage;
use crate::pages::onboarding::OnboardingPage;
use crate::pages::public_collection::PublicCollectionPage;
use crate::pages::register::RegisterPage;

/// SSR shell function — renders the outer HTML document
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <title>"Orchid Tracker"</title>
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta name="theme-color" content="#1b4332" />
                <AutoReload options=options.clone() />
                <HydrationScripts options=options.clone() />
                <HashedStylesheet id="leptos" options=options.clone() />
                <link rel="preconnect" href="https://fonts.googleapis.com" />
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
                <link href="https://fonts.googleapis.com/css2?family=DM+Serif+Display&family=Outfit:wght@300;400;500;600;700&display=swap" rel="stylesheet" />
            </head>
            <body>
                <App />
                <script>
                    "if ('serviceWorker' in navigator) { navigator.serviceWorker.register('/sw.js').catch(function(e) { console.warn('SW registration failed:', e); }); }"
                </script>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Router>
            <Routes fallback=|| "Page not found.">
                <Route path=path!("/") view=HomePage />
                <Route path=path!("/login") view=LoginPage />
                <Route path=path!("/register") view=RegisterPage />
                <Route path=path!("/onboarding") view=OnboardingPage />
                <Route path=path!("/u/:username") view=PublicCollectionPage />
            </Routes>
        </Router>
    }
}
