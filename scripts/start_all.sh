#!/usr/bin/env bash
# ============================================================
# Entropy Vault — Start All Services
# ============================================================
# Starts Mosquitto, Rust backend, Next.js frontend, and
# a test data publisher in one go.
#
# Usage:
#   ./scripts/start_all.sh
#
# Prerequisites:
#   1. Run ./scripts/setup_multichain.sh first
#   2. Install mosquitto: brew install mosquitto
#   3. Install frontend deps: cd frontend && npm install
#
# Press Ctrl+C to stop everything.
# ============================================================

set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# ── Configuration ────────────────────────────────────────────
# MQTT
MQTT_HOST="localhost"
MQTT_PORT="1883"
MQTT_USER="${MQTT_USER:-entropy}"
MQTT_PASS="${MQTT_PASS:-entropy123}"

# MultiChain — Docker-based setup (matches setup_multichain.sh)
CONTAINER_NAME="entropy-multichain"
CHAIN_NAME="entropy-chain"
MULTICHAIN_RPC_URL="${MULTICHAIN_RPC_URL:-http://localhost:6740}"
MULTICHAIN_RPC_USER="${MULTICHAIN_RPC_USER:-multichainrpc}"
MULTICHAIN_RPC_PASS="${MULTICHAIN_RPC_PASS:-entropy-vault-rpc-pass-2026}"

# Track child PIDs for cleanup
PIDS=()

cleanup() {
    echo ""
    echo "Shutting down all services..."
    for pid in "${PIDS[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
        fi
    done
    # Stop mosquitto if we started it
    if [ "${MOSQUITTO_STARTED:-false}" = "true" ]; then
        pkill -f "mosquitto -c" 2>/dev/null || true
    fi
    echo "All services stopped."
    exit 0
}

trap cleanup SIGINT SIGTERM EXIT

echo "============================================================"
echo "  Entropy Vault — Starting All Services"
echo "============================================================"
echo ""

# ── Step 1: Ensure MultiChain Docker container is running ────
echo "[1/5] MultiChain (Docker)..."
if ! docker info &> /dev/null; then
    echo "  WARNING: Docker is not running. Blockchain validation will fail."
    echo "  Start Docker Desktop, then run: ./scripts/setup_multichain.sh"
elif docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "  Container '$CONTAINER_NAME' is running."
elif docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "  Starting stopped container '$CONTAINER_NAME'..."
    docker start "$CONTAINER_NAME" > /dev/null
    sleep 3
    echo "  Container started."
else
    echo "  WARNING: Container '$CONTAINER_NAME' not found."
    echo "  Run ./scripts/setup_multichain.sh first to create it."
fi
echo "  RPC: $MULTICHAIN_RPC_URL (user: $MULTICHAIN_RPC_USER)"
echo ""

# ── Step 2: Start Mosquitto ──────────────────────────────────
echo "[2/5] Mosquitto MQTT broker..."
if pgrep -x mosquitto > /dev/null 2>&1; then
    echo "  Already running (using existing instance)."
    MOSQUITTO_STARTED=false
else
    # Create password file
    MQTT_PASSFILE="/tmp/entropy_vault_mqtt_passwd"
    if command -v mosquitto_passwd &> /dev/null; then
        mosquitto_passwd -b -c "$MQTT_PASSFILE" "$MQTT_USER" "$MQTT_PASS" 2>/dev/null
    else
        # Fallback: allow anonymous if mosquitto_passwd not available
        echo "  WARNING: mosquitto_passwd not found, starting without auth."
        MQTT_PASSFILE=""
    fi

    # Create config
    MQTT_CONF="/tmp/entropy_vault_mosquitto.conf"
    if [ -n "$MQTT_PASSFILE" ]; then
        cat > "$MQTT_CONF" <<EOF
listener $MQTT_PORT
password_file $MQTT_PASSFILE
allow_anonymous false
EOF
    else
        cat > "$MQTT_CONF" <<EOF
listener $MQTT_PORT
allow_anonymous true
EOF
        MQTT_USER=""
        MQTT_PASS=""
    fi

    mosquitto -c "$MQTT_CONF" -d
    MOSQUITTO_STARTED=true
    sleep 1
    echo "  Started on port $MQTT_PORT (user: ${MQTT_USER:-anonymous})"
fi
echo ""

# ── Step 3: Build & Start Rust Backend ───────────────────────
echo "[3/5] Rust backend (entropy-engine)..."
echo "  Building..."
(cd "$PROJECT_DIR/entropy-engine" && cargo build --quiet 2>&1) || {
    echo "  ERROR: cargo build failed!"
    exit 1
}
echo "  Starting OTP mode on port 3001..."

ENTROPY_MODE=otp \
MQTT_HOST="$MQTT_HOST" \
MQTT_PORT="$MQTT_PORT" \
MQTT_USER="$MQTT_USER" \
MQTT_PASS="$MQTT_PASS" \
MULTICHAIN_RPC_URL="$MULTICHAIN_RPC_URL" \
MULTICHAIN_RPC_USER="$MULTICHAIN_RPC_USER" \
MULTICHAIN_RPC_PASS="$MULTICHAIN_RPC_PASS" \
RUST_LOG=info \
"$PROJECT_DIR/entropy-engine/target/debug/entropy-engine" &
BACKEND_PID=$!
PIDS+=($BACKEND_PID)
sleep 2
echo "  Backend running (PID: $BACKEND_PID)"
echo ""

# ── Step 4: Start Next.js Frontend ───────────────────────────
echo "[4/5] Next.js frontend..."
(cd "$PROJECT_DIR/frontend" && npm run dev -- --port 3000) &
FRONTEND_PID=$!
PIDS+=($FRONTEND_PID)
sleep 3
echo "  Frontend running (PID: $FRONTEND_PID)"
echo ""

# ── Step 5: Start Test Data Publisher ────────────────────────
echo "[5/5] Test data publisher..."

# Publish loop in background
(
    PUB_ARGS=""
    if [ -n "$MQTT_USER" ]; then
        PUB_ARGS="-u $MQTT_USER -P $MQTT_PASS"
    fi

    MSG_COUNT=0
    while true; do
        # Generate random 4-digit numbers
        N1=$((RANDOM % 9999))
        N2=$((RANDOM % 9999))
        N3=$((RANDOM % 9999))
        N4=$((RANDOM % 9999))
        N5=$((RANDOM % 9999))
        N6=$((RANDOM % 9999))

        # Every 5th message, include MAC address
        if (( MSG_COUNT % 5 == 0 )); then
            mosquitto_pub -h "$MQTT_HOST" -p "$MQTT_PORT" $PUB_ARGS \
                -t "random/numbers" \
                -m "$N1,$N2,$N3,$N4,$N5,$N6,MAC:24:6F:28:AA:BB:CC" 2>/dev/null || true
        else
            mosquitto_pub -h "$MQTT_HOST" -p "$MQTT_PORT" $PUB_ARGS \
                -t "random/numbers" \
                -m "$N1,$N2,$N3,$N4,$N5,$N6" 2>/dev/null || true
        fi

        # Publish sensor params (21 columns of simulated data)
        TEMP=$(echo "scale=1; 20 + $((RANDOM % 150)) / 10" | bc)
        MOSFET=$((RANDOM % 4096))
        AX=$(echo "scale=2; ($((RANDOM % 200)) - 100) / 100" | bc)
        AY=$(echo "scale=2; ($((RANDOM % 200)) - 100) / 100" | bc)
        AZ=$(echo "scale=2; 9.8 + ($((RANDOM % 20)) - 10) / 100" | bc)
        GX=$(echo "scale=1; ($((RANDOM % 100)) - 50) / 10" | bc)
        GY=$(echo "scale=1; ($((RANDOM % 100)) - 50) / 10" | bc)
        GZ=$(echo "scale=1; ($((RANDOM % 100)) - 50) / 10" | bc)
        RED=$((RANDOM % 500))
        GREEN=$((RANDOM % 500))
        BLUE=$((RANDOM % 500))
        LUX=$((RANDOM % 1000))
        PM1=$(echo "scale=1; $((RANDOM % 500)) / 10" | bc)
        PM25=$(echo "scale=1; $((RANDOM % 800)) / 10" | bc)
        PM10=$(echo "scale=1; $((RANDOM % 1200)) / 10" | bc)
        C03=$((RANDOM % 3000))
        C05=$((RANDOM % 2000))
        C10=$((RANDOM % 500))
        C25=$((RANDOM % 100))
        C50=$((RANDOM % 30))
        C100=$((RANDOM % 10))

        mosquitto_pub -h "$MQTT_HOST" -p "$MQTT_PORT" $PUB_ARGS \
            -t "random/params" \
            -m "$MOSFET,$TEMP,$AX,$AY,$AZ,$GX,$GY,$GZ,$RED,$GREEN,$BLUE,$LUX,$PM1,$PM25,$PM10,$C03,$C05,$C10,$C25,$C50,$C100" 2>/dev/null || true

        MSG_COUNT=$((MSG_COUNT + 1))
        sleep 1
    done
) &
PUBLISHER_PID=$!
PIDS+=($PUBLISHER_PID)
echo "  Publisher running (PID: $PUBLISHER_PID)"
echo ""

# ── Ready ────────────────────────────────────────────────────
echo "============================================================"
echo "  All services running!"
echo ""
echo "  Frontend:   http://localhost:3000"
echo "  Backend:    http://localhost:3001"
echo "  MQTT:       $MQTT_HOST:$MQTT_PORT"
echo "  Blockchain: $MULTICHAIN_RPC_URL"
echo ""
echo "  Press Ctrl+C to stop everything."
echo "============================================================"

# Wait for all background processes
wait
