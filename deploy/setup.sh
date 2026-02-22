#!/usr/bin/env bash
# setup.sh — One-time server setup for OrchidTracker.
# Run as root or with sudo on a fresh Debian/Ubuntu server.

set -euo pipefail

APP_DIR="/opt/orchid-tracker"
SERVICE_USER="orchid"
REPO_URL="https://github.com/YOUR_USERNAME/OrchidTracker.git"  # <-- UPDATE THIS

echo "==> Creating service user '$SERVICE_USER'..."
if ! id "$SERVICE_USER" &>/dev/null; then
    useradd --system --shell /usr/sbin/nologin --home-dir "$APP_DIR" "$SERVICE_USER"
    echo "    Created user '$SERVICE_USER'"
else
    echo "    User '$SERVICE_USER' already exists"
fi

echo "==> Cloning repository to $APP_DIR..."
if [ ! -d "$APP_DIR/.git" ]; then
    git clone "$REPO_URL" "$APP_DIR"
else
    echo "    Repository already exists, pulling latest..."
    cd "$APP_DIR" && git pull --ff-only
fi

echo "==> Installing Rust toolchain for '$SERVICE_USER'..."
if ! sudo -u "$SERVICE_USER" bash -c 'command -v rustup' &>/dev/null; then
    sudo -u "$SERVICE_USER" bash -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
fi
sudo -u "$SERVICE_USER" bash -c 'source "$HOME/.cargo/env" && rustup target add wasm32-unknown-unknown'
sudo -u "$SERVICE_USER" bash -c 'source "$HOME/.cargo/env" && cargo install cargo-leptos'

echo "==> Creating data directories..."
mkdir -p "$APP_DIR/data/images"

echo "==> Setting up environment file..."
if [ ! -f "$APP_DIR/.env" ]; then
    cp "$APP_DIR/.env.example" "$APP_DIR/.env"
    echo "    IMPORTANT: Edit $APP_DIR/.env with production secrets!"
else
    echo "    .env already exists, skipping"
fi

echo "==> Setting ownership..."
chown -R "$SERVICE_USER:$SERVICE_USER" "$APP_DIR"

echo "==> Installing systemd service..."
cp "$APP_DIR/deploy/orchid-tracker.service" /etc/systemd/system/
systemctl daemon-reload
systemctl enable orchid-tracker

echo "==> Running initial build..."
sudo -u "$SERVICE_USER" bash -c "cd $APP_DIR && source \$HOME/.cargo/env && LEPTOS_TAILWIND_VERSION=v4.2.0 cargo leptos build --release"

echo "==> Starting service..."
systemctl start orchid-tracker

sleep 3
if systemctl is-active --quiet orchid-tracker; then
    echo ""
    echo "==> Setup complete! OrchidTracker is running."
    echo "    Status:  sudo systemctl status orchid-tracker"
    echo "    Logs:    journalctl -u orchid-tracker -f"
    echo "    Deploy:  cd $APP_DIR && ./deploy/deploy.sh"
else
    echo ""
    echo "==> Service failed to start. Check logs:"
    echo "    journalctl -u orchid-tracker -n 50 --no-pager"
    exit 1
fi
