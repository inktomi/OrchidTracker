# AGENTS.md — OrchidTracker

## Project Overview

OrchidTracker is a client-side-only (CSR) web application for managing an orchid collection. It compiles to WebAssembly and runs entirely in the browser with no backend server.

## Tech Stack

| Layer | Technology | Notes |
|---|---|---|
| Language | **Rust (latest stable)** | Target: `wasm32-unknown-unknown` |
| Framework | **Leptos 0.8+** (CSR only) | No SSR. Mounted via `leptos::mount::mount_to_body` |
| Styling | **Tailwind CSS v4** | Standalone CLI via Trunk — no Node.js |
| Sorting | **Rustywind** | CLI tool for sorting Tailwind classes (replaces Prettier plugin) |
| Build | **Trunk** | WASM bundler. Config in `Trunk.toml` |
| Deployment | GitHub Pages | Static site via `trunk build --release` |

## Architecture: The Elm Architecture (TEA)

All state management follows **The Elm Architecture** pattern: **Model → Update → View**.

The TEA triad lives in three files:

| TEA Layer | File | Leptos Mapping |
|---|---|---|
| **Model** | `src/model.rs` | `Model` struct (centralized state), `Msg` enum (all transitions), `Cmd` enum (side effects) |
| **Update** | `src/update.rs` | `update(&mut Model, Msg) -> Vec<Cmd>` (pure), `dispatch()` (runtime), `execute_cmd()` (side effects) |
| **View** | `src/app.rs` | `App` component: `signal(Model::init())`, `Memo` selectors, `view!` macro, event handlers dispatch `Msg` |

### How it works

1. **Model** — A single `signal(Model::init())` holds all app state. `Memo` selectors derive fine-grained reactive slices (e.g., `Memo::new(move |_| model.get().orchids.clone())`).
2. **Messages** — All state changes go through the `Msg` enum. Event handlers call `update::dispatch(set_model, model, Msg::SomeAction(...))`.
3. **Pure Update** — `update(&mut Model, Msg) -> Vec<Cmd>` is a pure function that modifies the model and returns commands. It is fully unit-testable with no browser dependencies.
4. **Commands** — Side effects (`Cmd::Persist`, `Cmd::SyncToGitHub`, `Cmd::ClearSyncAfterDelay`) are executed by `execute_cmd()`, which dispatches new messages when async work completes.

### Rules

- **All app state lives in `Model`** — no scattered signals in `app.rs`. Component-local form state (e.g., text inputs) may use local signals.
- **All state transitions go through `Msg`** — event handlers dispatch messages, never mutate the model directly.
- **`update()` is pure** — no side effects, no browser APIs, no async. Returns `Vec<Cmd>` to declare intent.
- **Side effects live in `execute_cmd()`** — async operations (GitHub sync, timers) dispatch new messages when complete.
- **No state mutation inside `view!`** — only signal/memo reads and event handler bindings.
- **Unidirectional data flow** — parent components pass data down via props; children communicate up via callback props (`on_close`, `on_update`, `on_delete`).
- **Use `Memo` selectors** — derive reactive slices from the model for fine-grained updates. Components receive `Memo<T>` instead of `ReadSignal<T>` for model-derived data.
- **Keep components focused** — each component owns its local signals and delegates to parent via callbacks for app-wide state changes.

## Rust Standards

### Write idiomatic, modern Rust

- Use **latest stable Rust** idioms: `let-else`, `if let` chains, pattern matching, iterators over manual loops.
- Prefer `impl Trait` in function signatures over concrete types or generics where appropriate.
- Use `derive` macros (`Clone`, `Debug`, `PartialEq`, `Serialize`, `Deserialize`) on all data types.
- Use enums with `Display` and `Serialize`/`Deserialize` for domain values (e.g., `LightRequirement`, `Placement`, `FitCategory`).
- Handle errors with `Result` and `Option` — no `unwrap()` in production paths. Use `unwrap_or`, `unwrap_or_else`, `?`, or `if let` / `let-else`.
- `unwrap()` and `expect()` are acceptable only in test code or truly unreachable branches (document why).
- No `unsafe` code.
- Run `cargo clippy --target wasm32-unknown-unknown` and resolve all warnings before committing.

### Leptos-specific conventions

- Use `#[component]` for all UI components. Components return `impl IntoView`.
- Use `ReadSignal` / `WriteSignal` for reactive state. Use `signal_local` only for non-`Send` types (e.g., `File`, `MediaStream`).
- Use `StoredValue` for data that doesn't change after initialization.
- Use `Effect::new` for side effects that react to signal changes.
- Use `on_cleanup` for resource cleanup (e.g., stopping media streams).
- Use `spawn_local` for async operations (API calls, IndexedDB).
- Prefer `NodeRef` for DOM element access over raw `web_sys` queries.

## Styling: Tailwind CSS

**All styling MUST use Tailwind utility classes.** No custom CSS files, no inline `style=""` attributes.

### Configuration

- **Input file:** `tailwind.css` — contains `@import "tailwindcss"`, `@source`, `@theme`, and `@layer base`
- **Source scanning:** `@source "./src/**/*.rs"` tells Tailwind v4 to scan Rust files for class names
- **Custom theme colors:** Defined in `@theme` block — use as `bg-primary`, `text-danger`, `border-shelf-high`, etc.
- **Base styles:** Global element defaults (body, label, input/textarea/select) are in `@layer base` so they don't need repeating

### Rules

- Apply Tailwind classes directly in `view!` macro `class=""` attributes.
- For repeated long class strings, define `const` strings at the top of the file (e.g., `const MODAL_OVERLAY: &str = "fixed inset-0 bg-black/50 ..."`).
- For dynamic classes, use full class strings per branch — not partial class concatenation. Tailwind's scanner must see complete class names:
  ```rust
  // GOOD — full class strings, scanner can find both
  class=move || if active { "bg-white text-primary font-bold" } else { "bg-transparent text-white" }

  // BAD — scanner can't detect dynamic concatenation
  class=format!("bg-{}", if active { "white" } else { "transparent" })
  ```
- Use `@theme` custom colors (`primary`, `danger`, `warning`, `shelf-high`, etc.) instead of hardcoded hex values.
- Never create a `.css` file. If a new base style is needed, add it to `@layer base` in `tailwind.css`.

## Project Structure

```
OrchidTracker/
├── Cargo.toml              # Rust dependencies
├── Trunk.toml              # Trunk build config (Tailwind CLI version)
├── index.html              # Entry point (Trunk processes this)
├── tailwind.css            # Tailwind input file (@theme, @layer base)
├── CNAME                   # GitHub Pages custom domain
└── src/
    ├── main.rs             # Entry point: mount App to body
    ├── lib.rs              # Module declarations
    ├── model.rs            # TEA Model (state), Msg (messages), Cmd (side effects)
    ├── update.rs           # TEA pure update function + dispatch + command execution
    ├── app.rs              # TEA View: root component, Memo selectors, event wiring
    ├── orchid.rs           # Domain model: Orchid, Placement, LightRequirement, etc.
    ├── db.rs               # IndexedDB operations (image blob storage via Rexie)
    ├── github.rs           # GitHub API sync (orchid JSON + image uploads)
    ├── error.rs            # Error types
    ├── components/
    │   ├── mod.rs
    │   ├── orchid_card.rs        # Grid view card
    │   ├── orchid_detail.rs      # Detail modal with history timeline
    │   ├── cabinet_table.rs      # Placement/table view with drag-and-drop
    │   ├── add_orchid_form.rs    # Add orchid modal form
    │   ├── scanner.rs            # Camera + Gemini AI scanner modal
    │   ├── settings.rs           # Settings modal (GitHub, Gemini, temp unit)
    │   └── climate_dashboard.rs  # AC Infinity climate data display
    └── data/
        ├── orchids.json    # Default orchid data (bundled at compile time)
        └── climate.json    # Climate sensor data (bundled at compile time)
```

## Data Persistence

- **LocalStorage** (`gloo-storage`): Primary orchid data, settings (GitHub token, API keys, temp unit).
- **IndexedDB** (`rexie`): Image blob storage for offline-first photo capture.
- **GitHub API** (`reqwest`): Sync orchid JSON and images to a user-configured repository.

Data flows: UI signals → LocalStorage (auto-persist via `Effect`) → GitHub (explicit sync).

## Testing

### Before every commit or push

1. **`cargo clippy --target wasm32-unknown-unknown`** — zero warnings.
2. **`cargo test`** — all unit tests pass (domain logic in `orchid.rs`, serialization, etc.).
3. **`rustywind . --write`** — sort Tailwind classes.
4. **`cargo check --target wasm32-unknown-unknown`** — full WASM compilation check.
5. **`trunk build`** — confirm Tailwind compiles and WASM bundles correctly.

### Testing guidelines

- Write unit tests for all domain logic, data transformations, and serialization/deserialization.
- Test files live alongside their modules using `#[cfg(test)] mod tests { ... }`.
- Test edge cases: empty collections, missing optional fields, enum variant round-trips.
- Component/integration testing via `trunk serve` + manual verification for UI behavior (modals, drag-drop, camera, sync).
- Do NOT commit code that has `cargo clippy` warnings or failing tests.

## Key Dependencies

| Crate | Purpose |
|---|---|
| `leptos` (CSR) | Reactive UI framework |
| `serde` / `serde_json` | Serialization for all data types |
| `gloo-storage` | LocalStorage wrapper |
| `gloo-file` | File API (image upload) |
| `gloo-timers` | Async timers |
| `rexie` | IndexedDB wrapper |
| `reqwest` (WASM) | HTTP client for GitHub/Gemini APIs |
| `chrono` (wasm-bindgen) | Date/time handling |
| `web-sys` / `js-sys` | Raw browser API bindings |
| `base64` / `sha2` / `hex` | Encoding for GitHub API |

## Common Patterns

### Modal components
All modals follow the same structure: overlay → content → header (title + close) → body. Use shared `const` strings for the common classes:
```rust
const MODAL_OVERLAY: &str = "fixed inset-0 bg-black/50 flex justify-center items-center z-[1000]";
const MODAL_HEADER: &str = "flex justify-between items-center mb-4 border-b border-gray-200 pb-2";
const CLOSE_BTN: &str = "bg-gray-400 text-white border-none py-2 px-3 rounded cursor-pointer hover:bg-gray-500";
```

### Dispatching messages (TEA Update)
Event handlers in the view dispatch messages through the pure update function:
```rust
// In app.rs — thin callback wrappers
let on_add = move |orchid| update::dispatch(set_model, model, Msg::AddOrchid(orchid));
let on_delete = move |id: u64| {
    // UI concerns (confirm dialog) stay in the view layer
    if let Some(window) = web_sys::window() {
        if !window.confirm_with_message("Delete?").unwrap_or(false) { return; }
    }
    update::dispatch(set_model, model, Msg::DeleteOrchid(id));
};

// Inline in view! macro
on:click=move |_| update::dispatch(set_model, model, Msg::TriggerSync)
```

### Callback props
Child-to-parent communication uses closure props. Components don't know about `Msg` — they call callbacks that the parent wires to dispatch:
```rust
#[component]
pub fn MyComponent(
    on_close: impl Fn() + 'static + Copy + Send + Sync,
    on_update: impl Fn(Orchid) + 'static + Copy + Send + Sync,
) -> impl IntoView { ... }
```

### Adding new features (TEA workflow)
1. Add fields to `Model` in `model.rs` if new state is needed
2. Add variants to `Msg` for the state transitions
3. Add match arms to `update()` in `update.rs` (pure logic only)
4. Add `Cmd` variants if side effects are needed, implement in `execute_cmd()`
5. Wire up the view in `app.rs` with `Memo` selectors and dispatch calls
6. Write unit tests for the new `update()` arms
