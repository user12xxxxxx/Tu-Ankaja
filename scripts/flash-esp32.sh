#!/usr/bin/env bash
# Flash the ESP32 with the entropy MQTT firmware.
# Usage: ./scripts/flash-esp32.sh [PORT]
#   PORT defaults to auto-detect

set -e

SKETCH="firmware/mqtt/entropy_mqtt.ino"
BOARD="esp32:esp32:esp32"  # Generic ESP32 DevKit

# Auto-detect port if not provided
if [ -n "$1" ]; then
    PORT="$1"
else
    PORT=$(arduino-cli board list 2>/dev/null | grep -i "esp32\|CP210\|CH340\|USB" | awk '{print $1}' | head -1)
    if [ -z "$PORT" ]; then
        echo "No ESP32 detected. Plug it in via USB, then re-run."
        echo "Or specify the port manually: $0 /dev/cu.usbserial-XXXX"
        echo ""
        echo "Available ports:"
        arduino-cli board list 2>/dev/null || ls /dev/cu.usb* /dev/tty.usb* 2>/dev/null
        exit 1
    fi
fi

echo "=== Entropy Vault — ESP32 Flasher ==="
echo "Sketch: $SKETCH"
echo "Board:  $BOARD"
echo "Port:   $PORT"
echo ""

# Install ESP32 core if needed
if ! arduino-cli core list 2>/dev/null | grep -q "esp32:esp32"; then
    echo "Installing ESP32 board core..."
    arduino-cli core install esp32:esp32 --additional-urls https://raw.githubusercontent.com/espressif/arduino-esp32/gh-pages/package_esp32_index.json
fi

# Install PubSubClient library if needed
if ! arduino-cli lib list 2>/dev/null | grep -q "PubSubClient"; then
    echo "Installing PubSubClient library..."
    arduino-cli lib install "PubSubClient"
fi

echo "Compiling..."
arduino-cli compile --fqbn "$BOARD" "$SKETCH"

echo "Uploading to $PORT..."
arduino-cli upload --fqbn "$BOARD" --port "$PORT" "$SKETCH"

echo ""
echo "Done! ESP32 is flashed and will connect to WiFi automatically."
echo "Run the engine with: ENTROPY_MODE=mqtt cargo run"
