#!/usr/bin/env bash
# deploy.sh — Download latest release from GitHub and restart the service.
# Run on the server: /opt/orchids/deploy/deploy.sh
# No build toolchain needed — binary is pre-built in CI.

set -euo pipefail

REPO="inktomi/OrchidTracker"
APP_DIR="/opt/orchids"
SERVICE_USER="orchid"
SERVICE="orchid-tracker"
HEALTH_URL="http://localhost:3000"

echo "==> Fetching latest release info..."
DOWNLOAD_URL=$(curl -sf "https://api.github.com/repos/$REPO/releases/latest" \
    | grep '"browser_download_url"' \
    | head -1 \
    | sed 's/.*"\(https[^"]*\)".*/\1/')

if [ -z "$DOWNLOAD_URL" ]; then
    echo "ERROR: Could not find release asset URL."
    exit 1
fi

TARBALL="/tmp/orchid-tracker-release.tar.gz"

echo "==> Downloading $DOWNLOAD_URL ..."
curl -fL -o "$TARBALL" "$DOWNLOAD_URL"

STAGING=$(mktemp -d)
echo "==> Unpacking release..."
tar xzf "$TARBALL" -C "$STAGING"
rm -f "$TARBALL"

echo "==> Installing to $APP_DIR ..."
sudo -u "$SERVICE_USER" cp "$STAGING/orchid-tracker" "$APP_DIR/target/release/orchid-tracker"
sudo -u "$SERVICE_USER" cp "$STAGING/hash.txt" "$APP_DIR/target/release/hash.txt"
sudo -u "$SERVICE_USER" rsync -a --delete "$STAGING/site/" "$APP_DIR/target/site/"
sudo -u "$SERVICE_USER" rsync -a "$STAGING/migrations/" "$APP_DIR/migrations/"
rm -rf "$STAGING"

echo "==> Restarting service..."
sudo systemctl restart "$SERVICE"

echo "==> Waiting for startup..."
for i in {1..30}; do
    if curl -sf "$HEALTH_URL" > /dev/null 2>&1; then
        echo "==> Deploy successful! Service is running."
        exit 0
    fi
    sleep 1
done

echo "==> WARNING: Health check failed. Check logs with:"
echo "    journalctl -u $SERVICE -n 50 --no-pager"
exit 1
