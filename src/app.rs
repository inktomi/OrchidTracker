use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use serde::{Deserialize, Serialize};

use crate::pages::home::HomePage;
use crate::pages::login::LoginPage;
use crate::pages::register::RegisterPage;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClimateData {
    pub name: String,
    pub type_str: Option<String>,
    pub temperature: f64,
    pub humidity: f64,
    pub vpd: f64,
    pub updated: String,
}

/// SSR shell function — renders the outer HTML document
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html>
            <head>
                <meta charset="utf-8" />
                <title>"Orchid Tracker"</title>
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta name="theme-color" content="#1b4332" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <link rel="preconnect" href="https://fonts.googleapis.com" />
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
                <link href="https://fonts.googleapis.com/css2?family=DM+Serif+Display&family=Outfit:wght@300;400;500;600;700&display=swap" rel="stylesheet" />
            </head>
            <body>
                <App />
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
            </Routes>
        </Router>
    }
}
