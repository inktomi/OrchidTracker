# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Summary

OrchidTracker is a **client-side-only (CSR) Rust/WebAssembly** web app for managing an orchid collection. It runs entirely in the browser with no backend server. Deployed at https://orchids.reef.fish/ via GitHub Pages.

## Build & Development Commands

```bash
# Dev server with live reload (http://127.0.0.1:8080)
trunk serve

# Production build
trunk build --release --public-url /

# Run tests
cargo test

# Lint (zero warnings required)
cargo clippy --target wasm32-unknown-unknown

# Sort Tailwind classes
rustywind . --write

# Full WASM compilation check
cargo check --target wasm32-unknown-unknown
```

**Pre-commit checklist** (run all before committing):
1. `cargo clippy --target wasm32-unknown-unknown` — zero warnings
2. `cargo test` — all pass
3. `rustywind . --write` — sort Tailwind classes
4. `trunk build` — verify CSS and WASM bundle

## Architecture: The Elm Architecture (TEA)

All state management follows **Model → Update → View** in three files:

| TEA Layer | File | Contains |
|-----------|------|----------|
| **Model** | `src/model.rs` | `Model` struct (all app state), `Msg` enum (transitions), `Cmd` enum (side effects) |
| **Update** | `src/update.rs` | Pure `update()` function, `dispatch()` runtime, `execute_cmd()` for async side effects |
| **View** | `src/app.rs` | Root `App` component, `Memo` selectors, event handlers that dispatch `Msg` |

**Key rules:**
- `update()` is **pure** — no side effects, no browser APIs, no async. Returns `Vec<Cmd>`.
- All state lives in `Model` — components use `Memo<T>` selectors for fine-grained reactivity.
- Side effects are declared as `Cmd` variants and executed in `execute_cmd()`.
- Children communicate to parents via callback props (`on_close`, `on_update`), never via `Msg` directly.

**Adding a new feature:**
1. Add fields to `Model` in `model.rs`
2. Add `Msg` variants for state transitions
3. Add match arms in `update()` (pure logic only)
4. Add `Cmd` variants if side effects needed, implement in `execute_cmd()`
5. Wire the view in `app.rs` with `Memo` selectors and dispatch calls

## Tech Stack

- **Rust** targeting `wasm32-unknown-unknown`
- **Leptos 0.8+** (CSR only, no SSR) — reactive framework with signals/memos
- **Tailwind CSS v4** via Trunk's standalone CLI (no Node.js)
- **Trunk** — WASM bundler and dev server
- **Storage:** LocalStorage (`gloo-storage`) + IndexedDB (`rexie`) for images
- **APIs:** GitHub REST/LFS sync (`reqwest`), Google Gemini AI scanning

## Styling Rules

- **Tailwind utility classes only** — no custom CSS files, no inline `style=""`.
- Custom theme colors defined in `@theme` block in `tailwind.css` (`primary`, `danger`, `warning`, `shelf-high`, etc.).
- Dynamic classes must use **full class strings per branch** (Tailwind scanner needs complete names):
  ```rust
  // GOOD
  class=move || if active { "bg-white text-primary" } else { "bg-gray-100 text-gray-500" }
  // BAD — scanner can't detect
  class=format!("bg-{}", if active { "white" } else { "gray-100" })
  ```
- Repeated class strings: define `const` at top of file.

## Rust Conventions

- No `unwrap()` in production paths — use `unwrap_or`, `?`, `if let`, `let-else`.
- Zero `cargo clippy` warnings required.
- Use `signal_local` only for non-`Send` types (e.g., `File`, `MediaStream`).
- Use `spawn_local` for async operations.
- Use `NodeRef` for DOM access over raw `web_sys` queries.

## Key Source Files

| File | Purpose |
|------|---------|
| `src/orchid.rs` | Domain model: `Orchid`, `Placement`, `LightRequirement`, `FitCategory` |
| `src/db.rs` | IndexedDB image blob storage |
| `src/github.rs` | GitHub API sync (REST + LFS for images) |
| `src/error.rs` | `AppError` enum with variants for all failure modes |
| `src/components/*.rs` | UI components (cards, detail modal, scanner, settings, cabinet table) |
| `tailwind.css` | Tailwind config (`@theme` colors, `@layer base` defaults, `@source` scanning) |

## Data Persistence

Three tiers: in-memory signals → LocalStorage (auto-persist via `Effect`) → GitHub (explicit sync button).

## Further Reference

See `AGENTS.md` for detailed architecture documentation, common patterns (modals, dispatching, callbacks), and the complete project structure.
