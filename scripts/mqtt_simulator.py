#!/usr/bin/env python3
"""
MQTT Entropy Simulator — publishes random bytes to a Mosquitto broker.

This simulates a hardware entropy source (like an ESP32) sending random
bytes over WiFi/MQTT. Use this to test the entropy-engine in MQTT mode
without actual hardware.

Requirements:
    pip install paho-mqtt

Usage:
    # Start Mosquitto first:
    mosquitto -d

    # Run the simulator:
    python3 scripts/mqtt_simulator.py

    # In another terminal, run the engine in MQTT mode:
    ENTROPY_MODE=mqtt cargo run

    # Custom broker/topic:
    python3 scripts/mqtt_simulator.py --host 192.168.1.50 --topic entropy-vault/raw
"""

import argparse
import os
import time
import struct

try:
    import paho.mqtt.client as mqtt
except ImportError:
    print("Error: paho-mqtt not installed.")
    print("Install it with: python3 -m pip install paho-mqtt")
    raise SystemExit(1)


def make_mqtt_client(client_id: str):
    """Create an MQTT client compatible with both paho-mqtt v1 and v2."""
    try:
        # paho-mqtt v2: requires CallbackAPIVersion as first argument.
        return mqtt.Client(
            mqtt.CallbackAPIVersion.VERSION2,
            client_id=client_id,
        )
    except (AttributeError, TypeError):
        # paho-mqtt v1: no CallbackAPIVersion.
        return mqtt.Client(client_id=client_id)


def generate_entropy_bytes(length: int) -> bytes:
    """Generate random bytes using OS entropy (urandom)."""
    return os.urandom(length)


def main():
    parser = argparse.ArgumentParser(description="MQTT Entropy Simulator")
    parser.add_argument("--host", default="localhost", help="MQTT broker host")
    parser.add_argument("--port", type=int, default=1883, help="MQTT broker port")
    parser.add_argument("--topic", default="entropy-vault/raw", help="MQTT topic")
    parser.add_argument("--interval", type=float, default=0.5, help="Seconds between publishes")
    parser.add_argument("--bytes", type=int, default=64, help="Bytes per message")
    args = parser.parse_args()

    client = make_mqtt_client("entropy-simulator")

    print(f"Connecting to MQTT broker at {args.host}:{args.port}...")
    try:
        client.connect(args.host, args.port, keepalive=60)
    except ConnectionRefusedError:
        print(f"Error: Cannot connect to {args.host}:{args.port}")
        print("Is Mosquitto running? Start it with: mosquitto -d")
        raise SystemExit(1)

    client.loop_start()

    print(f"Publishing {args.bytes} bytes every {args.interval}s to '{args.topic}'")
    print("Press Ctrl+C to stop.\n")

    total_bytes = 0
    msg_count = 0

    try:
        while True:
            payload = generate_entropy_bytes(args.bytes)
            result = client.publish(args.topic, payload, qos=1)
            result.wait_for_publish()

            total_bytes += len(payload)
            msg_count += 1

            if msg_count % 10 == 0:
                print(f"  Published {msg_count} messages ({total_bytes} bytes total)")

            time.sleep(args.interval)

    except KeyboardInterrupt:
        print(f"\nStopped. Published {msg_count} messages ({total_bytes} bytes total).")

    finally:
        client.loop_stop()
        client.disconnect()


if __name__ == "__main__":
    main()
