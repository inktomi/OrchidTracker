#!/usr/bin/env bash
# deploy.sh — Pull latest code, rebuild, and restart the service.
# Run on the server: cd /opt/orchid-tracker && ./deploy/deploy.sh

set -euo pipefail

APP_DIR="/opt/orchid-tracker"
SERVICE="orchid-tracker"
HEALTH_URL="http://localhost:3000"

cd "$APP_DIR"

echo "==> Pulling latest changes..."
git pull --ff-only

echo "==> Building release..."
LEPTOS_TAILWIND_VERSION=v4.2.0 cargo leptos build --release

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
