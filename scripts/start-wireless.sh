#!/usr/bin/env bash
# Start the full wireless entropy pipeline:
#   1. Connect to remote MQTT broker (ESP32 publishes random/numbers + random/params)
#   2. Start the Rust entropy engine in OTP mode
#
# Usage: ./scripts/start-wireless.sh

set -e

# MQTT broker config
MQTT_HOST="${MQTT_HOST:-10.175.74.41}"
MQTT_PORT="${MQTT_PORT:-1883}"
MQTT_USER="${MQTT_USER:-Navalogy}"
MQTT_PASS="${MQTT_PASS:-Man_Mera_Mandir}"

echo "=== Entropy Vault — Wireless Mode ==="
echo ""
echo "MQTT broker: $MQTT_HOST:$MQTT_PORT (user: $MQTT_USER)"
echo "Topics:      random/numbers, random/params, random/MAC"
echo ""

# Start the Rust engine in OTP mode (subscribes to random/numbers + random/params)
echo "Starting entropy engine..."
echo "Dashboard: http://localhost:3000 (run 'cd frontend && npm run dev' separately)"
echo "API:       http://localhost:3001"
echo ""
cd "$(dirname "$0")/../entropy-engine"
ENTROPY_MODE=otp \
  MQTT_HOST="$MQTT_HOST" \
  MQTT_PORT="$MQTT_PORT" \
  MQTT_USER="$MQTT_USER" \
  MQTT_PASS="$MQTT_PASS" \
  SKIP_BLOCKCHAIN=1 \
  RUST_LOG=info \
  cargo run
