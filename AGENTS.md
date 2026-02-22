# AGENTS.md — OrchidTracker

## Project Overview

OrchidTracker is a full-stack web application for managing an orchid collection. It uses Leptos with Server-Side Rendering (SSR) and Axum as the backend server. Data is persisted to a SurrealDB instance.

## Tech Stack

| Layer | Technology | Notes |
|---|---|---|
| Language | **Rust (latest stable)** | Target: `wasm32-unknown-unknown` (frontend) and native (backend) |
| Framework | **Leptos 0.8+** (SSR) | Server-side rendered with hydration. Axum backend. |
| Styling | **Tailwind CSS v4** | Standalone CLI via Leptos build |
| Sorting | **Rustywind** | CLI tool for sorting Tailwind classes (replaces Prettier plugin) |
| Backend | **Axum / Tokio** | HTTP server and async runtime |
| Database | **SurrealDB** | Document/Graph database |
| Server Fns| `#[server]` | Leptos Server Functions for RPC calls |

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

- **Database** (`surrealdb`): Primary storage for users, orchids, zones, and climate readings.
- **Sessions** (`tower-sessions`): User authentication state stored in SurrealDB.
- **Server Functions** (`#[server]`): RPC calls from the frontend execute database queries on the backend.
- **Local State**: Limited to UI-specific preferences (e.g., dark mode).

## Observability & Tracing

All logging and distributed tracing are handled via the `tracing` ecosystem and exported to **Axiom**.

### Tracing Standards
- **Use `tracing::*` macros**: Use `info!`, `warn!`, `error!`, etc. across the entire stack (both frontend and backend). Do not use the `log` crate or `eprintln!`/`println!`.
- **Instrument Server Functions**: Every Leptos server function (`#[server]`) MUST be instrumented with `#[tracing::instrument(level = "info", skip_all)]` to ensure a named child span is created for the RPC.
- **Instrument Background Tasks**: Any `tokio::spawn` loops or async tasks outside the HTTP request lifecycle MUST be instrumented with `.instrument(tracing::info_span!("task_name"))` (requires `use tracing::Instrument;`).
- **Axum Router**: The main HTTP router utilizes `tower_http::trace::TraceLayer` to generate root spans for all incoming HTTP requests.
- **Axiom Integration**: The backend server uses `tracing-axiom` to asynchronously batch and ship all spans to Axiom. This requires `AXIOM_TOKEN` and `AXIOM_DATASET` environment variables to be set in production. If not present, tracing safely falls back to standard output.

## Testing

### Philosophy: Deep Testing
Testing is not merely a box to check; it is the primary mechanism for **validation** and **regression prevention**. A test suite should give you absolute confidence to refactor and deploy. Tests must execute the code in ways that matter, evaluating real outcomes, not superficial execution paths.

- **Depth over Breadth**: It is better to have fewer tests that deeply validate behavior than many tests that only check if a function executes without panicking.
- **Unit Testing**: Validate domain logic, state transitions (TEA update loops), and complex calculations in isolation.
- **Integration Testing**: Validate that components work together. For database interactions, we use **SurrealDB in-memory (`surrealdb` with `kv-mem` feature)** to execute real queries against a real database engine, avoiding brittle mocks.
- **End-to-End (E2E) Testing**: Validate the user journey. We use **Playwright** (configured via `end2end-cmd = "npx playwright test"`) to ensure the full stack (Leptos frontend + Axum backend + SurrealDB) functions correctly from the user's perspective.

### Effective Test Criteria
1. **Meaningful Assertions**: Assert against specific data states, not just `is_ok()`. Verify that the database state actually changed or that the UI reflects the expected text.
2. **Edge Cases**: Actively test failure modes, missing data, boundary conditions, and invalid inputs.
3. **Coverage**: Focus coverage on business logic and complex state management (`update.rs`, `db.rs`, `server_fns/`).

### Anti-Patterns to Avoid
- **Shallow Checks**: Writing tests that only verify a function returns without error, without checking *what* it returned or what side effects occurred.
- **Mocks Misuse**: Over-mocking database calls or HTTP requests. Whenever possible, use the real in-memory SurrealDB instance or a local test server to ensure your queries and schemas are actually correct. Mocking should be a last resort for external APIs only.

### Before every commit or push

1. **`cargo clippy --target wasm32-unknown-unknown`** — zero warnings.
2. **`cargo test`** — all tests pass (domain logic, in-memory DB queries, etc.).
3. **`rustywind . --write`** — sort Tailwind classes.
4. **`cargo check --target wasm32-unknown-unknown`** — full WASM compilation check.
5. **`trunk build`** — confirm Tailwind compiles and WASM bundles correctly.
6. **`npx playwright test`** — confirm E2E user flows remain intact.

### Testing guidelines

- Write unit tests for all domain logic, data transformations, and serialization/deserialization.
- Test files live alongside their modules using `#[cfg(test)] mod tests { ... }` or in the `tests/` directory for integration.
- Component/integration testing via `trunk serve` + manual verification for UI behavior (modals, drag-drop, camera, sync), supplemented by Playwright.
- Do NOT commit code that has `cargo clippy` warnings or failing tests.

## Key Dependencies

| Crate | Purpose |
|---|---|
| `leptos` | Reactive UI framework (SSR + Hydration) |
| `axum` | Backend HTTP server |
| `tokio` | Async runtime |
| `surrealdb` | Database client |
| `serde` / `serde_json` | Serialization for all data types |
| `reqwest` | HTTP client for external APIs |
| `chrono` | Date/time handling |
| `tower-sessions` | Session management |
| `argon2` | Password hashing |

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

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

## Issue Tracking

This project uses **bd (beads)** for issue tracking.
Run `bd prime` for workflow context, or install hooks (`bd hooks install`) for auto-injection.

**Quick reference:**
- `bd ready` - Find unblocked work
- `bd create "Title" --type task --priority 2` - Create issue
- `bd close <id>` - Complete work
- `bd sync` - Sync with git (run at session end)

For full workflow details: `bd prime`
