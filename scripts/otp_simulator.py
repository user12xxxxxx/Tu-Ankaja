#!/usr/bin/env python3
"""
OTP MQTT Simulator — publishes realistic sensor data to 3 topics.

Simulates the MYOSA board: random/numbers, random/params, random/MAC
Use this when the remote broker is unavailable.

Usage:
    mosquitto -d
    python3 scripts/otp_simulator.py
"""

import os
import time
import random
import json

try:
    import paho.mqtt.client as mqtt
except ImportError:
    print("Error: paho-mqtt not installed.")
    print("Install it with: python3 -m pip install paho-mqtt")
    raise SystemExit(1)


def make_client(client_id):
    try:
        return mqtt.Client(mqtt.CallbackAPIVersion.VERSION2, client_id=client_id)
    except (AttributeError, TypeError):
        return mqtt.Client(client_id=client_id)


def random_sensor_csv():
    """Generate a CSV row matching the 21 MYOSA sensor columns."""
    mosfet = random.randint(500, 4095)
    temp = round(random.uniform(32.0, 40.0), 2)
    acc_x = round(random.uniform(-2.0, 2.0), 3)
    acc_y = round(random.uniform(-2.0, 2.0), 3)
    acc_z = round(random.uniform(8.0, 11.0), 3)
    gyro_x = round(random.uniform(-5.0, 5.0), 3)
    gyro_y = round(random.uniform(-5.0, 5.0), 3)
    gyro_z = round(random.uniform(-5.0, 5.0), 3)
    red = random.randint(100, 800)
    green = random.randint(100, 800)
    blue = random.randint(100, 800)
    lux = round(random.uniform(10.0, 500.0), 1)
    pm1 = round(random.uniform(1.0, 30.0), 1)
    pm25 = round(random.uniform(2.0, 50.0), 1)
    pm10 = round(random.uniform(3.0, 80.0), 1)
    c03 = random.randint(100, 2000)
    c05 = random.randint(50, 1000)
    c10 = random.randint(10, 500)
    c25 = random.randint(1, 100)
    c50 = random.randint(0, 30)
    c100 = random.randint(0, 10)
    return f"{mosfet},{temp},{acc_x},{acc_y},{acc_z},{gyro_x},{gyro_y},{gyro_z},{red},{green},{blue},{lux},{pm1},{pm25},{pm10},{c03},{c05},{c10},{c25},{c50},{c100}"


def random_numbers():
    """Generate 4 comma-separated 4-digit random numbers."""
    nums = [random.randint(1000, 9999) for _ in range(4)]
    return ",".join(str(n) for n in nums)


def main():
    client = make_client("otp-simulator")

    print("Connecting to local MQTT broker...")
    try:
        client.connect("localhost", 1883, keepalive=60)
    except ConnectionRefusedError:
        print("Error: Cannot connect to localhost:1883")
        print("Start Mosquitto first: mosquitto -d")
        raise SystemExit(1)

    client.loop_start()

    mac = "4C:C3:82:36:81:04"
    mac_json = json.dumps({"mac": mac})

    # Send MAC once at start
    client.publish("random/MAC", mac_json, qos=1)
    print(f"Published MAC: {mac}")

    print("Publishing to random/numbers and random/params every 500ms")
    print("Press Ctrl+C to stop.\n")

    msg_count = 0
    try:
        while True:
            nums = random_numbers()
            params = random_sensor_csv()

            client.publish("random/numbers", nums, qos=1)
            client.publish("random/params", params, qos=1)

            msg_count += 1
            if msg_count % 10 == 0:
                print(f"  {msg_count} messages sent  |  numbers: {nums}")

            time.sleep(0.5)
    except KeyboardInterrupt:
        print(f"\nStopped after {msg_count} messages.")
    finally:
        client.loop_stop()
        client.disconnect()


if __name__ == "__main__":
    main()
