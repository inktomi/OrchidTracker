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
./target/release/orchid-tracker reset-password --username <user> --password <new-password>
```

### Reprocess Plants with AI

Re-run AI species analysis on all plants for a given user. Useful after integrating new data sources (e.g., Andy's Orchids nursery data) to update temperature/humidity tolerances and seasonal care data.

```bash
# Preview what would be processed
./target/release/orchid-tracker reprocess-plants --user inktomi --dry-run

# Process with defaults (5 per batch, 30s delay between batches)
./target/release/orchid-tracker reprocess-plants --user inktomi

# Custom batch settings
./target/release/orchid-tracker reprocess-plants --user inktomi --batch-size 3 --delay-secs 60
```

Only AI-derived fields are updated (temp ranges, humidity, seasonal care, conservation status, native region, light requirement, water frequency). User-set fields like name, notes, placement, pot info, and fertilizer settings are preserved.

## Running the Server

Pre-built release binaries are published via GitHub Actions — no Rust toolchain needed on the server.

### Requirements

- Linux (x86_64)
- [SurrealDB](https://surrealdb.com/) v3 running and accessible
- (Optional) [Gemini](https://ai.google.dev/) and/or Claude API keys for AI plant identification

### Install

1. Download the latest release tarball from [GitHub Releases](https://github.com/inktomi/OrchidTracker/releases/latest).

2. Create an install directory and unpack:
   ```bash
   sudo mkdir -p /opt/orchids/target/release /opt/orchids/target/site /opt/orchids/data/images
   cd /opt/orchids
   tar xzf /path/to/orchid-tracker-*.tar.gz
   mv orchid-tracker target/release/
   mv site target/
   # migrations/ extracts in place
   ```

3. Create an `.env` file (see `.env.example` for all options):
   ```bash
   cat > /opt/orchids/.env <<'EOF'
   SURREAL_URL=ws://127.0.0.1:8000
   SURREAL_NS=orchidtracker
   SURREAL_DB=orchidtracker
   SURREAL_USER=root
   SURREAL_PASS=changeme
   IMAGE_STORAGE_PATH=/opt/orchids/data/images
   SESSION_SECRET=generate-a-long-random-string-at-least-64-chars
   LEPTOS_SITE_ADDR=0.0.0.0:3000
   EOF
   ```

4. Run directly:
   ```bash
   cd /opt/orchids && target/release/orchid-tracker
   ```

### Systemd Service (recommended)

1. Create a service user:
   ```bash
   sudo useradd -r -s /usr/sbin/nologin orchid
   sudo chown -R orchid:orchid /opt/orchids
   ```

2. Install the service file:
   ```bash
   sudo cp deploy/orchid-tracker.service /etc/systemd/system/
   sudo systemctl daemon-reload
   sudo systemctl enable --now orchid-tracker
   ```

3. Check status:
   ```bash
   sudo systemctl status orchid-tracker
   journalctl -u orchid-tracker -f
   ```

### Updating

Run the deploy script to download the latest release and restart the service:

```bash
/opt/orchids/deploy/deploy.sh
```

## Technologies

- [Leptos 0.8](https://github.com/leptos-rs/leptos) — SSR + hydration, `#[server]` functions
- [Axum 0.8](https://github.com/tokio-rs/axum) — HTTP server and routing
- [SurrealDB 3](https://surrealdb.com/) — database (remote WebSocket connection)
- [Tailwind CSS v4](https://tailwindcss.com/) — utility-first styling via cargo-leptos integration
- [Gemini / Claude APIs](https://ai.google.dev/) — AI plant identification with automatic fallback
- [tower-sessions](https://crates.io/crates/tower-sessions) — session-based authentication
- [argon2](https://crates.io/crates/argon2) — password hashing
