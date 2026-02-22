#!/usr/bin/env bash
# deploy.sh — Pull latest code, rebuild, and restart the service.
# Run on the server: /opt/orchids/deploy/deploy.sh
# No sudo needed — script elevates only for systemctl.

set -euo pipefail

APP_DIR="/opt/orchids"
SERVICE_USER="orchid"
SERVICE="orchid-tracker"
HEALTH_URL="http://localhost:3000"

echo "==> Pulling latest changes..."
sudo -u "$SERVICE_USER" git -C "$APP_DIR" pull --ff-only

echo "==> Building release..."
sudo -u "$SERVICE_USER" bash -c "source '$APP_DIR/.cargo/env' && cd '$APP_DIR' && LEPTOS_TAILWIND_VERSION=v4.2.0 cargo leptos build --release"

echo "==> Restarting service..."
sudo systemctl restart "$SERVICE"

echo "==> Waiting for startup..."
sleep 3

if curl -sf "$HEALTH_URL" > /dev/null 2>&1; then
    echo "==> Deploy successful! Service is running."
else
    echo "==> WARNING: Health check failed. Check logs with:"
    echo "    journalctl -u $SERVICE -n 50 --no-pager"
    exit 1
fi
