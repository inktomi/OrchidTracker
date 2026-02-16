# Orchid Tracker (Rust/Leptos)

A simple web application to track your orchids and their care requirements, built with Rust and Leptos.

## Features

-   **Dashboard:** View all your orchids in a grid.
-   **Add Orchid:** Form to add new orchids with details (Name, Species, Watering Frequency, Light Requirement, Notes).
-   **Delete Orchid:** Remove orchids from the list.
-   **Persistence:** Data is saved locally in your browser (LocalStorage).

## Prerequisites

-   [Rust](https://www.rust-lang.org/tools/install)
-   `trunk` (WASM build tool): `cargo install trunk`
-   `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`

## Local Development

1.  Clone the repository.
2.  Run the development server:
    ```bash
    trunk serve
    ```
3.  Open `http://127.0.0.1:8080` in your browser.

## Deployment (GitHub Pages)

This project includes a GitHub Action workflow to automatically deploy to GitHub Pages.

1.  Push your code to the `main` branch.
2.  Go to your repository settings on GitHub.
3.  Under **Pages**, ensure the source is set to `gh-pages` branch (created by the action).
4.  Your site will be live at `http://orchids.reef.fish/`.

## Technologies

-   [Leptos](https://github.com/leptos-rs/leptos) - A full-stack, isomorphic Rust web framework.
-   [Trunk](https://trunkrs.dev/) - Build, bundle & ship your Rust WASM application to the web.
-   [Tailwind CSS](https://tailwindcss.com/) (Optional, currently using custom CSS).
