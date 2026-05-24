#!/usr/bin/env bash
# ============================================================
# MultiChain Setup Script for Entropy Vault (Docker)
# ============================================================
# This script:
#   1. Pulls and runs MultiChain in a Docker container
#   2. Creates a blockchain called "entropy-chain"
#   3. Creates a "valid-macs" stream for MAC address validation
#   4. Seeds it with a test MAC address
#   5. Prints RPC credentials for the backend
#
# Prerequisites:
#   - Docker Desktop installed and running
#
# Usage:
#   chmod +x scripts/setup_multichain.sh
#   ./scripts/setup_multichain.sh
# ============================================================

set -euo pipefail

CONTAINER_NAME="entropy-multichain"
CHAIN_NAME="entropy-chain"
STREAM_NAME="valid-macs"
TEST_MAC="24:6F:28:AA:BB:CC"
RPC_PORT=6740
RPC_USER="multichainrpc"
RPC_PASS="entropy-vault-rpc-pass-2026"

echo "=== Entropy Vault — MultiChain Setup (Docker) ==="
echo ""

# ── Step 1: Check Docker ─────────────────────────────────────
if ! command -v docker &> /dev/null; then
    echo "ERROR: Docker is not installed."
    echo "Install Docker Desktop from: https://www.docker.com/products/docker-desktop/"
    exit 1
fi

if ! docker info &> /dev/null; then
    echo "ERROR: Docker daemon is not running."
    echo "Please start Docker Desktop and re-run this script."
    exit 1
fi

echo "Docker is running."
echo ""

# ── Step 2: Start or reuse container ─────────────────────────
if docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    if docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
        echo "Container '$CONTAINER_NAME' is already running."
    else
        echo "Container '$CONTAINER_NAME' exists but stopped. Starting..."
        docker start "$CONTAINER_NAME"
        sleep 3
    fi
else
    echo "Creating and starting MultiChain container..."

    # Use Ubuntu base image and install MultiChain inside it
    docker run -d \
        --name "$CONTAINER_NAME" \
        -p "${RPC_PORT}:${RPC_PORT}" \
        --platform linux/amd64 \
        ubuntu:22.04 \
        bash -c "
            apt-get update -qq && apt-get install -y -qq wget ca-certificates > /dev/null 2>&1
            cd /tmp
            wget -q https://www.multichain.com/download/multichain-2.3.3.tar.gz
            tar -xzf multichain-2.3.3.tar.gz
            cp multichain-2.3.3/multichaind multichain-2.3.3/multichain-cli multichain-2.3.3/multichain-util /usr/local/bin/

            # Create the blockchain
            multichain-util create $CHAIN_NAME \
                -default-rpc-port=$RPC_PORT \
                -anyone-can-connect=true \
                -anyone-can-send=true \
                -anyone-can-receive=true \
                -anyone-can-create=true

            # Set RPC credentials
            echo 'rpcuser=$RPC_USER' > /root/.multichain/$CHAIN_NAME/multichain.conf
            echo 'rpcpassword=$RPC_PASS' >> /root/.multichain/$CHAIN_NAME/multichain.conf
            echo 'rpcallowip=0.0.0.0/0' >> /root/.multichain/$CHAIN_NAME/multichain.conf
            echo 'rpcbind=0.0.0.0' >> /root/.multichain/$CHAIN_NAME/multichain.conf

            # Start the daemon in foreground (keeps container alive)
            multichaind $CHAIN_NAME -daemon

            # Keep container running
            tail -f /dev/null
        "

    echo "Waiting for MultiChain to initialize (this takes ~60-120s on first run)..."

    # Wait for the daemon to be ready
    MAX_WAIT=180
    WAITED=0
    while [ $WAITED -lt $MAX_WAIT ]; do
        if docker exec "$CONTAINER_NAME" multichain-cli "$CHAIN_NAME" getinfo &> /dev/null; then
            break
        fi
        sleep 3
        WAITED=$((WAITED + 3))
        echo "  Waiting... (${WAITED}s)"
    done

    if [ $WAITED -ge $MAX_WAIT ]; then
        echo ""
        echo "Timed out waiting for MultiChain. Checking logs..."
        docker logs --tail 30 "$CONTAINER_NAME"
        echo ""
        echo "Try: docker logs $CONTAINER_NAME"
        exit 1
    fi
fi

echo ""

# ── Step 3: Verify daemon is responding ──────────────────────
echo "Checking MultiChain daemon..."
docker exec "$CONTAINER_NAME" multichain-cli "$CHAIN_NAME" getinfo > /dev/null 2>&1 || {
    echo "  Daemon not ready yet, waiting 5s..."
    sleep 5
}

CHAIN_INFO=$(docker exec "$CONTAINER_NAME" multichain-cli "$CHAIN_NAME" getinfo 2>/dev/null || echo "ERROR")
if echo "$CHAIN_INFO" | grep -q "ERROR"; then
    echo "ERROR: MultiChain daemon is not responding."
    echo "Check: docker logs $CONTAINER_NAME"
    exit 1
fi
echo "MultiChain daemon is running."
echo ""

# ── Step 4: Create the valid-macs stream ─────────────────────
echo "Creating stream '$STREAM_NAME'..."
EXISTING=$(docker exec "$CONTAINER_NAME" multichain-cli "$CHAIN_NAME" liststreams "$STREAM_NAME" 2>/dev/null || echo "")

if echo "$EXISTING" | grep -q "$STREAM_NAME"; then
    echo "  Stream '$STREAM_NAME' already exists."
else
    docker exec "$CONTAINER_NAME" multichain-cli "$CHAIN_NAME" create stream "$STREAM_NAME" false
    echo "  Stream created."
    sleep 1
fi

# Subscribe to the stream
docker exec "$CONTAINER_NAME" multichain-cli "$CHAIN_NAME" subscribe "$STREAM_NAME" 2>/dev/null || true
echo "  Subscribed to stream."
echo ""

# ── Step 5: Seed with test MAC address ───────────────────────
echo "Publishing test MAC address: $TEST_MAC"
docker exec "$CONTAINER_NAME" multichain-cli "$CHAIN_NAME" \
    publish "$STREAM_NAME" "$TEST_MAC" \
    "{\"json\":{\"mac\":\"$TEST_MAC\",\"status\":\"valid\",\"added\":\"setup-script\"}}"
echo "  Test MAC published."
echo ""

# ── Step 6: Verify lookup ────────────────────────────────────
echo "Verifying MAC lookup..."
RESULT=$(docker exec "$CONTAINER_NAME" multichain-cli "$CHAIN_NAME" \
    liststreamkeyitems "$STREAM_NAME" "$TEST_MAC" 2>/dev/null)
if echo "$RESULT" | grep -q "$TEST_MAC"; then
    echo "  SUCCESS: MAC '$TEST_MAC' found in blockchain."
else
    echo "  WARNING: MAC lookup returned unexpected result:"
    echo "  $RESULT"
fi
echo ""

# ── Step 7: Print credentials ────────────────────────────────
echo "============================================================"
echo "  MultiChain Setup Complete!"
echo "============================================================"
echo ""
echo "  Container:  $CONTAINER_NAME"
echo "  RPC URL:    http://localhost:$RPC_PORT"
echo "  RPC User:   $RPC_USER"
echo "  RPC Pass:   $RPC_PASS"
echo "  Test MAC:   $TEST_MAC (valid)"
echo ""
echo "  Backend env vars:"
echo "    export MULTICHAIN_RPC_URL=http://localhost:$RPC_PORT"
echo "    export MULTICHAIN_RPC_USER=$RPC_USER"
echo "    export MULTICHAIN_RPC_PASS=$RPC_PASS"
echo ""
echo "  Add more valid MACs:"
echo "    docker exec $CONTAINER_NAME multichain-cli $CHAIN_NAME \\"
echo "      publish $STREAM_NAME \"XX:XX:XX:XX:XX:XX\" \\"
echo "      '{\"json\":{\"mac\":\"XX:XX:XX:XX:XX:XX\",\"status\":\"valid\"}}'"
echo ""
echo "  Stop:   docker stop $CONTAINER_NAME"
echo "  Start:  docker start $CONTAINER_NAME"
echo "  Remove: docker rm -f $CONTAINER_NAME"
echo "============================================================"
