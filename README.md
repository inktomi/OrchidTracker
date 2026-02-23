# Orchid Tracker

A full-stack Rust web application for managing an orchid collection with multi-user authentication, AI-powered plant identification, climate monitoring, and seasonal care tracking.

**Live at:** [orchids.reef.fish](https://orchids.reef.fish)

## Features

- **Collection Management:** Dashboard with card and table views for your plants, including watering schedules, fertilizer tracking, and repotting history.
- **AI Plant Identification:** Scan a photo or search by name to identify species using Gemini/Claude with automatic fallback. Integrates Andy's Orchids nursery data for refined care recommendations.
- **Climate Monitoring:** Growing zones with live temperature/humidity readings from hardware sensors (WeatherFlow Tempest, AC Infinity) and manual entries. Alerts when conditions drift outside plant tolerances.
- **Seasonal Care:** Automatic rest/bloom period tracking with adjusted watering and fertilizer schedules per hemisphere.
- **Habitat Weather:** Tracks weather in each plant's native habitat for comparison with your growing conditions.
- **Multi-User Auth:** Session-based authentication with per-user data isolation.
- **Public Collections:** Optionally share your collection via a public URL.
- **Push Notifications:** Web push alerts for overdue watering and climate warnings.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- `cargo-leptos`: `cargo install cargo-leptos`
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [SurrealDB](https://surrealdb.com/) v3

## Local Development

1. Clone the repository.
2. Copy `.env.example` to `.env` and configure your SurrealDB connection and API keys.
3. Run the development server:
   ```bash
   cargo leptos watch
   ```
4. Open `http://0.0.0.0:3000` in your browser.

## Testing

```bash
# Run all tests
cargo test --features ssr

# Check SSR target
cargo check --features ssr

# Check WASM target
cargo check --features hydrate --target wasm32-unknown-unknown
```

- **Unit Tests:** Located alongside source in `src/` files — domain models, helper functions, serde roundtrips.
- **Integration Tests:** Located in `tests/` — DB-backed tests using SurrealDB's in-memory backend.

## CLI Commands

The server binary includes administrative CLI commands.

### Reset Password

```bash
cargo run --features ssr -- reset-password --username <user> --password <new-password>
```

### Reprocess Plants with AI

Re-run AI species analysis on all plants for a given user. Useful after integrating new data sources (e.g., Andy's Orchids nursery data) to update temperature/humidity tolerances and seasonal care data.

```bash
# Preview what would be processed
cargo run --features ssr -- reprocess-plants --user inktomi --dry-run

# Process with defaults (5 per batch, 30s delay between batches)
cargo run --features ssr -- reprocess-plants --user inktomi

# Custom batch settings
cargo run --features ssr -- reprocess-plants --user inktomi --batch-size 3 --delay-secs 60
```

Only AI-derived fields are updated (temp ranges, humidity, seasonal care, conservation status, native region, light requirement, water frequency). User-set fields like name, notes, placement, pot info, and fertilizer settings are preserved.

## Deployment

Self-hosted on a Linux server. A deploy script handles pull, build, and service restart:

```bash
/opt/orchids/deploy/deploy.sh
```

See `deploy/` for the systemd service file and setup script.

## Technologies

- [Leptos 0.8](https://github.com/leptos-rs/leptos) — SSR + hydration, `#[server]` functions
- [Axum 0.8](https://github.com/tokio-rs/axum) — HTTP server and routing
- [SurrealDB 3](https://surrealdb.com/) — database (remote WebSocket connection)
- [Tailwind CSS v4](https://tailwindcss.com/) — utility-first styling via cargo-leptos integration
- [Gemini / Claude APIs](https://ai.google.dev/) — AI plant identification with automatic fallback
- [tower-sessions](https://crates.io/crates/tower-sessions) — session-based authentication
- [argon2](https://crates.io/crates/argon2) — password hashing
