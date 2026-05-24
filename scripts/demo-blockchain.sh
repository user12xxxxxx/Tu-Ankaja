#!/usr/bin/env bash
# Blockchain Device Authentication Demo
# Run this for your video — shows the full validation flow with colors

CYAN='\033[0;36m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
WHITE='\033[1;37m'
DIM='\033[2m'
BOLD='\033[1m'
RESET='\033[0m'

MAC="4C:C3:82:36:81:04"
FAKE_MAC="AA:BB:CC:DD:EE:FF"

clear

echo ""
echo -e "${BOLD}${WHITE}========================================${RESET}"
echo -e "${BOLD}${CYAN}  ENTROPY VAULT — DEVICE AUTHENTICATION${RESET}"
echo -e "${BOLD}${WHITE}========================================${RESET}"
echo ""
echo -e "${DIM}Blockchain: MultiChain (entropy-chain)${RESET}"
echo -e "${DIM}Stream:     valid-macs${RESET}"
echo ""
sleep 2

# Step 1: Show registered devices
echo -e "${YELLOW}[1] Querying blockchain for registered devices...${RESET}"
sleep 1
echo ""
echo -e "${DIM}> multichain-cli entropy-chain liststreamitems valid-macs${RESET}"
sleep 1

ITEMS=$(docker exec entropy-multichain multichain-cli entropy-chain liststreamitems valid-macs 2>/dev/null)
COUNT=$(echo "$ITEMS" | grep -c '"keys"')
echo ""
echo -e "  ${WHITE}Registered devices on chain: ${GREEN}${COUNT}${RESET}"
echo ""
sleep 2

# Step 2: Validate REAL MAC
echo -e "${YELLOW}[2] ESP32 connects and sends MAC: ${WHITE}${MAC}${RESET}"
sleep 1
echo -e "    ${DIM}MQTT topic: random/MAC${RESET}"
echo -e "    ${DIM}Payload: {\"mac\":\"${MAC}\"}${RESET}"
echo ""
sleep 1

echo -e "    Checking blockchain..."
sleep 1

RESULT=$(docker exec entropy-multichain multichain-cli entropy-chain liststreamkeyitems valid-macs "$MAC" 2>/dev/null)
CONF=$(echo "$RESULT" | grep -o '"confirmations" : [0-9]*' | grep -o '[0-9]*')
TXID=$(echo "$RESULT" | grep -o '"txid" : "[^"]*"' | head -1 | cut -d'"' -f4)
STATUS=$(echo "$RESULT" | grep -o '"status" : "[^"]*"' | head -1 | cut -d'"' -f4)

if [ -n "$CONF" ]; then
    echo ""
    echo -e "    ${GREEN}${BOLD}DEVICE AUTHENTICATED${RESET}"
    echo ""
    echo -e "    ${WHITE}MAC Address:     ${CYAN}${MAC}${RESET}"
    echo -e "    ${WHITE}Status:          ${GREEN}${STATUS}${RESET}"
    echo -e "    ${WHITE}Confirmations:   ${GREEN}${CONF}${RESET}"
    echo -e "    ${WHITE}Transaction:     ${DIM}${TXID}${RESET}"
    echo ""
    echo -e "    ${GREEN}>>> All random numbers from this device: ACCEPTED${RESET}"
else
    echo -e "    ${RED}DEVICE NOT FOUND${RESET}"
fi
echo ""
sleep 3

# Step 3: Try FAKE MAC
echo -e "${YELLOW}[3] Attacker spoofs a fake device: ${RED}${FAKE_MAC}${RESET}"
sleep 1
echo -e "    ${DIM}MQTT topic: random/MAC${RESET}"
echo -e "    ${DIM}Payload: {\"mac\":\"${FAKE_MAC}\"}${RESET}"
echo ""
sleep 1

echo -e "    Checking blockchain..."
sleep 1

FAKE_RESULT=$(docker exec entropy-multichain multichain-cli entropy-chain liststreamkeyitems valid-macs "$FAKE_MAC" 2>/dev/null)
FAKE_COUNT=$(echo "$FAKE_RESULT" | grep -c '"keys"')

echo ""
if [ "$FAKE_COUNT" -eq 0 ]; then
    echo -e "    ${RED}${BOLD}DEVICE REJECTED${RESET}"
    echo ""
    echo -e "    ${WHITE}MAC Address:     ${RED}${FAKE_MAC}${RESET}"
    echo -e "    ${WHITE}Status:          ${RED}NOT REGISTERED${RESET}"
    echo -e "    ${WHITE}On-chain entries: ${RED}0${RESET}"
    echo ""
    echo -e "    ${RED}>>> All random numbers from this device: DISCARDED${RESET}"
fi

echo ""
sleep 2
echo -e "${BOLD}${WHITE}========================================${RESET}"
echo -e "${DIM}  Blockchain is immutable — registered${RESET}"
echo -e "${DIM}  MACs cannot be tampered with.${RESET}"
echo -e "${BOLD}${WHITE}========================================${RESET}"
echo ""
