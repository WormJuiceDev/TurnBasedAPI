#!/bin/sh

set -e

APP_NAME="friendbox"
APP_DIR="/root/app"
BIN_PATH="/usr/local/bin/${APP_NAME}"
LOG_PATH="/tmp/friendbox.log"
ENV_PATH="${APP_DIR}/friendbox.env"
SERVICE_SOURCE="${APP_DIR}/freebsd/friendbox"
SERVICE_TARGET="/usr/local/etc/rc.d/friendbox"
RUNNER_SOURCE="${APP_DIR}/freebsd/run_friendbox.sh"
RUNNER_TARGET="/usr/local/bin/run_friendbox.sh"

echo "--- Stopping Current FriendBox Server ---"
service friendbox stop 2>/dev/null || true
killall -9 "${APP_NAME}" 2>/dev/null || true
killall -9 daemon 2>/dev/null || true

echo "--- Compiling FriendBox Backend ---"
cd "${APP_DIR}"
cargo build --release

echo "--- Installing FriendBox Backend ---"
rm -f "${BIN_PATH}"
cp "target/release/${APP_NAME}" "${BIN_PATH}"
chmod +x "${BIN_PATH}"

echo "--- Installing FriendBox Service ---"
if [ -f "${SERVICE_SOURCE}" ]; then
    mkdir -p "$(dirname "${SERVICE_TARGET}")"
    cp "${SERVICE_SOURCE}" "${SERVICE_TARGET}"
    sed -i '' -e 's/\r$//' "${SERVICE_TARGET}" 2>/dev/null || true
    chmod +x "${SERVICE_TARGET}"
else
    echo "Missing service file: ${SERVICE_SOURCE}"
    exit 1
fi

if [ -f "${RUNNER_SOURCE}" ]; then
    cp "${RUNNER_SOURCE}" "${RUNNER_TARGET}"
    sed -i '' -e 's/\r$//' "${RUNNER_TARGET}" 2>/dev/null || true
    chmod +x "${RUNNER_TARGET}"
else
    echo "Missing runner file: ${RUNNER_SOURCE}"
    exit 1
fi

if [ -f "${ENV_PATH}" ]; then
    sed -i '' -e 's/\r$//' "${ENV_PATH}" 2>/dev/null || true
else
    echo "--- No friendbox.env found; service will start with default environment ---"
fi

echo "--- Enabling FriendBox Autostart ---"
sysrc friendbox_enable=YES

echo "--- Starting FriendBox Server ---"
rm -f "${LOG_PATH}" /var/run/friendbox.pid
service friendbox start

echo "--- Status Check ---"
sleep 2
sockstat -4 -l | grep 3000 || {
    echo "FriendBox is not listening on port 3000. Recent log output:"
    tail -50 "${LOG_PATH}" 2>/dev/null || true
    exit 1
}

echo "--- FriendBox Deployment Complete ---"
