# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Summary

OrchidTracker is a **full-stack Rust web app** (Leptos SSR + hydrated WASM client) for managing an orchid collection with multi-user authentication. It uses SurrealDB for data storage, Axum for the HTTP server, and server-side Gemini AI proxying.

**Deployment target:** Self-hosted Linux server at velamen.app via OPNsense firewall.

## Build & Development Commands

```bash
# Dev server with live reload (http://0.0.0.0:3000)
cargo leptos watch

# Production build (server binary + WASM bundle)
cargo leptos build --release

# Run tests
cargo test

# Check SSR target
cargo check --features ssr

# Check hydrate/WASM target
cargo check --features hydrate --target wasm32-unknown-unknown

# Sort Tailwind classes
rustywind . --write
```

**Pre-commit checklist** (run all before committing):
1. `RUSTFLAGS="-D warnings" cargo check --features ssr` — SSR compiles (warnings are errors, matching CI)
2. `RUSTFLAGS="-D warnings" cargo check --features hydrate --target wasm32-unknown-unknown` — WASM compiles
3. `cargo test` — all pass
4. `rustywind . --write` — sort Tailwind classes
5. `cargo leptos build` — verify full build

## Architecture

### Dual-target build
- **SSR (`ssr` feature):** Server binary with Axum, SurrealDB, auth, image storage
- **Hydrate (`hydrate` feature):** WASM client with reactive UI, browser APIs
- Built with `cargo-leptos` which compiles both targets

### TEA Pattern (UI State Only)
Model/Update/View (Elm Architecture) handles UI-only state:

| TEA Layer | File | Contains |
|-----------|------|----------|
| **Model** | `src/model.rs` | `Model` (UI state), `Msg` (transitions), `Cmd` (side effects) |
| **Update** | `src/update.rs` | Pure `update()`, `dispatch()`, `execute_cmd()` |
| **View** | `src/pages/home.rs` | Main page with orchid CRUD via server functions |

### Server Functions
Data operations use Leptos `#[server]` functions that run on the server:
- `src/server_fns/auth.rs` — login, register, logout, get_current_user
- `src/server_fns/orchids.rs` — CRUD orchids + log entries
- `src/server_fns/scanner.rs` — Gemini AI proxy
- `src/server_fns/images.rs` — Multipart image upload (custom Axum handler)

### Feature Gating
- Browser APIs (`web_sys`, `wasm_bindgen`) gated behind `#[cfg(feature = "hydrate")]`
- Server modules (`db`, `auth`, `config`) gated behind `#[cfg(feature = "ssr")]`
- SurrealDB `bind()` requires owned values (not references) due to `'static` bound

## Tech Stack

- **Rust** — dual target: native server + `wasm32-unknown-unknown`
- **Leptos 0.8** — SSR + hydration, `#[server]` functions
- **Axum 0.8** — HTTP server, routing, middleware
- **SurrealDB 2** — database (remote WebSocket connection)
- **tower-sessions** — session-based authentication
- **argon2** — password hashing
- **Tailwind CSS v4.2** — via cargo-leptos built-in support (pinned via `LEPTOS_TAILWIND_VERSION` env var)
- **reqwest** — server-side HTTP client (Gemini API)

## Styling Rules

- **Tailwind utility classes only** — no custom CSS files, no inline `style=""`.
- Custom theme colors defined in `@theme` block in `tailwind.css`.
- Dynamic classes must use **full class strings per branch**.
- Repeated class strings: define `const` at top of file.

## Rust Conventions

- No `unwrap()` in production paths — use `unwrap_or`, `?`, `if let`, `let-else`.
- Use `signal_local` only for non-`Send` types (e.g., `File`, `MediaStream`).
- Use `spawn_local` for async operations on the client.
- Use `NodeRef` for DOM access over raw `web_sys` queries.
- Feature-gate browser APIs behind `#[cfg(feature = "hydrate")]`.

## Key Source Files

| File | Purpose |
|------|---------|
| `src/main.rs` | Axum server entry point (SSR) |
| `src/lib.rs` | Module declarations + hydrate entry |
| `src/app.rs` | SSR shell + Router with auth routes |
| `src/orchid.rs` | Domain model: `Orchid`, `Placement`, `LightRequirement` |
| `src/db.rs` | SurrealDB connection pool + migration runner (SSR) |
| `src/auth.rs` | Password hashing + session helpers (SSR) |
| `src/config.rs` | `AppConfig` from env vars (SSR) |
| `src/error.rs` | `AppError` enum |
| `src/pages/*.rs` | Page components (home, login, register) |
| `src/server_fns/*.rs` | Server functions (auth, orchids, scanner, images) |
| `src/components/*.rs` | UI components (cards, detail, scanner, settings, cabinet) |
| `migrations/*.surql` | SurrealDB schema migrations |

## Component Testing

Leptos 0.8's `.to_html()` method renders components to HTML strings in native Rust tests — no WASM or browser needed.

### Pattern
```rust
#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use leptos::prelude::*;
    use leptos::reactive::owner::Owner;
    use crate::test_helpers::test_orchid_with_care;

    #[test]
    fn test_component_hides_button_when_read_only() {
        let owner = Owner::new();
        owner.with(|| {
            let (sig, set_sig) = signal(test_orchid_with_care());
            let html = view! { <MyComponent orchid=sig set_orchid=set_sig read_only=true /> }.to_html();
            assert!(!html.contains("Delete"), "Should be hidden in read-only mode");
        });
    }
}
```

### Key points
- Tests go **inside** the component file in a `#[cfg(all(test, feature = "ssr"))]` module (access to private sub-components)
- Run with `cargo test --features ssr`
- Shared test builders in `src/test_helpers.rs`: `test_orchid()`, `test_orchid_with_care()`, `test_orchid_seasonal()`
- **Must wrap test body in `Owner::new().with(|| { ... })`** — `sandboxed-arenas` feature requires an active arena for signals
- For callbacks: provide no-op functions (`fn noop(_: String) {}`)
- Tests verify **rendered HTML structure**, not interactions (use Playwright for click/form testing)

## Data Persistence

Server-side SurrealDB with per-user data isolation. Images stored on server filesystem.

## Configuration

Environment variables in `.env` (see `.env.example`):
- `SURREAL_URL`, `SURREAL_NS`, `SURREAL_DB`, `SURREAL_USER`, `SURREAL_PASS`
- `IMAGE_STORAGE_PATH`, `GEMINI_API_KEY`, `GEMINI_MODEL`, `SESSION_SECRET`
