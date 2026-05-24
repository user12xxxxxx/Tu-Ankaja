---
publishDate: 2026-05-14T00:00:00Z
title: TU Ankaja: Electronic Noise Powered and MYOSA-integrated Sensor Based Random Number Generator.
excerpt: A multi-sensor hardware entropy collector built on an ESP32 based MYOSA Motherboard and MYOSA sensor and an external circuit, feeding a cryptographic engine over MQTT with randomly generated numbers.
image: /assets/images/Tu-Ankaja/chaotic-box-inside-full.jpg
tags:
  - IoT
  - Rust
  - MOSFET
  - MQTT
  - MYOSA-kit
  - TRNgs
---

![IoT](https://img.shields.io/badge/IoT-blue) ![Rust](https://img.shields.io/badge/Factory--Safety-orange) ![MOSFET](https://img.shields.io/badge/MOSFET-green) ![MQTT](https://img.shields.io/badge/MQTT-purple) ![MYOSA-kit](https://img.shields.io/badge/MYOSA-kit-red) ![Randomness](https://img.shields.io/badge/Randomness-61DAFB)

> Turn raw sensor noise into cryptographically secure random numbers, visualized in real time.

---

## Acknowledgements

Built by **Team TU Ankaja** for the IEEE MYOSA Innovation Challenge, organized by the **IEEE Sensors Council**. We would like to thank the MYOSA organizers and IEEE Sensors Council for providing the MYOSA development platform and the opportunity to explore true random number generation. We also acknowledge the guidance and support provided by our mentor **Dr. Rupam Goswami**, Assistant Professor, Department of ECE, Tezpur University, throughout this project.

**Team Members:**

| Name | Department | Role |
|------|-----------|------|
| Yash Sharma | B.Tech 4th Semester, ECE | Lead Developer — Rust backend, MQTT pipeline, frontend dashboard |
| Nautesh Kanojiya | B.Tech 4th Semester, ECE | Hardware Design — MOSFET noise circuit, chaotic box construction, sensor wiring |
| Nabajyoti Das | B.Tech 4th Semester, ECE | Firmware — ESP32 MQTT publisher, sensor data collection, BCD clamping logic |
| Hritima Rabha | B.Tech 4th Semester, ECE | Testing & Documentation — Integration tests, README, demo scripts |

---

## Overview

**TU Ankaja** is an innovative hardware-software system designed to generate true random numbers by combining natural stochastic electronic noise with unpredictable physical parameters.

**What problem does it solve?**
- Overcomes the predictability of purely algorithmic pseudo-random number generators.
- Provides a **low analog computational cost** solution for capturing random electronic fluctuations.
- Serves as a customized hardware source for true randomness, perfectly **tailored for low-to-moderate priority security applications.**

**Hardware Pinout:**

| Component / Function | MYOSA Mother Board Pin |
|---|---|
| DAC output / MOSFET Gate input | GPIO 25 |
| ADC input / MOSFET Drain output | GPIO 32 |
| Motor Driver ENB | GPIO 23 |
| Motor Driver IN3 | GPIO 26 |
| Motor Driver IN4 | GPIO 27 |
| LED 1 | GPIO 5 |
| LED 2 | GPIO 18 |
| LED 3 | GPIO 19 |
| UART Tx (PMS5003) | GPIO 16 |
| UART Rx (PMS5003) | GPIO 17 |
| I2C SDA | GPIO 21 |
| I2C SCL | GPIO 22 |


**Key Features:**
* **Analog Noise Generation:** Utilizes an IRF540N n-channel MOSFET as a switch to generate high-frequency noise signals.
* **Multi-Sensor Aggregation:** Captures physical parameters like Electronic noise using MOSFET, RGB light, ambient light, temperature, gyroscope data (in x, y, z), air particles, simultaneously.
* **Custom Chaotic Environment:** Employs a physical mirrored box with rotating LEDs, moving discs, and agitated air particles to create a highly dynamic sensory input.
* **Digit-Picking Algorithm:** Uses an array-based system to process sensor data into true random numbers.
* **Validation of the RNG using NIST Suite:** The random numbers generated using the prototype were tested using the NIST Suite, and the system was found to pass __/15 tests, which is considered to be excellent.

---

## Demo / Examples

### Images

<p align="center">
  <img src="/assets/images/Tu-Ankaja/mosfet-circuit-closeup.jpg" width="800"><br/>
  <i>Close-up of the MOSFET noise circuit on perfboard (2k ohm pull-up + 82 ohm gate resistor) mounted on the MYOSA motherboard with 0.96" OLED display</i>
</p>

<p align="center">
  <img src="/assets/images/Tu-Ankaja/chaotic-box-inside-full.jpg" width="800"><br/>
  <i>Inside the chaotic box — mirror foil walls, PMS5003 particle sensor (blue, top-left), APDS9960 light sensor, MYOSA Motherboard, motor axle with LED disc, motor driver (L298N), MPU6050, and fan for particle agitation</i>
</p>

<p align="center">
  <img src="/assets/images/Tu-Ankaja/chaotic-box-sensors.jpg" width="800"><br/>
  <i>Close-up of internal wiring — PMS5003 particle sensor (top), MYOSA Motherboard (red PCB), MPU6050 accelerometer/gyroscope, and linear voltage regulator</i>
</p>

<p align="center">
  <img src="/assets/images/Tu-Ankaja/chaotic-box-motor.jpg" width="800"><br/>
  <i>Motor assembly with rotating disc and RGB LEDs for visual disturbance of the APDS9960 sensor, mirrored interior walls visible</i>
</p>

<p align="center">
  <img src="/assets/images/Tu-Ankaja/dashboard-raw-data.png" width="800"><br/>
  <i>Raw Data Viewer — live MQTT sensor graphs for MOSFET noise, color (RGB), ambient light, temperature, accelerometer, and gyroscope across 21 channels</i>
</p>

<p align="center">
  <img src="/assets/images/Tu-Ankaja/dashboard-otp.png" width="800"><br/>
  <i>OTP Generator — hardware-backed one-time passwords generated from MOSFET noise + SHA-256, with generation history</i>
</p>

### Videos

<video controls width="100%">
  <source src="/myosa-demo.mp4" type="video/mp4">
</video>

---

## Features (Detailed)

### **1. Analog Noise Generator Circuit**

Electronic noise in MOSFETs is naturally stochastic. Circuit Design: The Drain (D) terminal is connected to +3.3V using a 2k ohm pull-up resistor. Input: A random value from 0 to 255 using `dacWrite(25, esp_random() & 0xFF)` function is applied at the Gate (G) from the DAC pin 25 of MYOSA Motherboard through a small gate resistor 82 ohm. Output: The resulting random signal is harvested from the Drain terminal and fed directly to the 12-bit ADC pin 32 of the MYOSA motherboard.

### **2. Chaotic Hardware Environment**

To gather unpredictable digital data, a 45 cm x 45 cm box with a rough mirrored inner wall houses multiple stimuli:

- **Visual Disturbance:** A motor rotates colored LEDs and sweeps a disc around the APDS9960 sensor to trigger random RGB and gesture data.
- **Particle Agitation:** A PC fan continuously blows air inside the box, scattering particles for the PMS5003 sensor to detect.
- **Environmental Metrics:** External BMP180 and CCS811 sensors gather ambient temperature, pressure, humidity, and volatile organic compounds to add extra environmental entropy.

### **3. Digital Processing & BCD Clamping**

- **Data Collection:** Sensors communicate via I2C and UART (PMS5003), sending 8-bit digital data packets.
- **Array Initialization:** The system accumulates two sets of 8-bit data into a 16-bit variable for each sensor.
- **Random Bit Selection:** A software "BitPicker" randomly selects bits from across the sensor arrays.
- **BCD Clamping:** The 16 random bits are grouped, and a modulo operator (%10) limits the decimal equivalent of the chunks to 9 (preventing hex values up to 15), finalizing the 16-bit random output.

### **4. NIST Complient**

- **NIST SP-800 22**
<p align="center">
  <img src="/assets/images/Tu-Ankaja/Nist.png" width="800"><br/>
  <i>NIST SP 800-90B entropy estimation results</i>
</p>

- **NIST SP-800 90b**


```plaintext
./ea_non_iid -i -v clean.bin 8
Opening file: 'clean.bin' (SHA-256 hash 6f6f763f3cf419aebe4138ecd45f902c032e9d886d5b7f92fe613d569d741e88)
Loaded 500000 samples of 256 distinct 8-bit-wide symbols
Number of Binary Symbols: 4000000

*** Warning: data contains less than 1000000 samples ***


Running non-IID tests...

Running Most Common Value Estimate...
Bitstring MCV Estimate: mode = 2000846, p-hat = 0.50021150000000003, p_u = 0.50085545734877057
        Most Common Value Estimate (bit string) = 0.997534 / 1 bit(s)
Literal MCV Estimate: mode = 2103, p-hat = 0.0042059999999999997, p_u = 0.0044417501069705847
        Most Common Value Estimate = 7.814656 / 8 bit(s)

Running Entropic Statistic Estimates (bit strings only)...
Bitstring Collision Estimate: X-bar = 2.5001853262836424, sigma-hat = 0.50000012191585219, p = 0.52040707920565799
        Collision Test Estimate (bit string) = 0.942288 / 1 bit(s)
Bitstring Markov Estimate: P_0 = 0.50021150000000003, P_1 = 0.49978849999999997, P_0,0 = 0.5004090771648978, P_0,1 = 0.4995909228351022, P_1,0 = 0.50001400592450607, P_1,1 = 0.49998599407549393, p_max = 3.2617554867162069e-39
        Markov Test Estimate (bit string) = 0.998825 / 1 bit(s)
Bitstring Compression Estimate: X-bar = 5.2177154580149221, sigma-hat = 1.0146812433751566, p = 0.024829087768112323
        Compression Test Estimate (bit string) = 0.888637 / 1 bit(s)

Running Tuple Estimates...
Bitstring t-Tuple Estimate: t = 18, p-hat_max = 0.5252200659857931316476, p_u = 0.5258632036900274298085
Bitstring LRS Estimate: u = 19, v = 41, p-hat = 0.49998108984273857, p_u = 0.50062504724865993
        T-Tuple Test Estimate (bit string) = 0.927241 / 1 bit(s)
Literal t-Tuple Estimate: t = 1, p-hat_max = 0.004205999999999999999829, p_u = 0.004441750106970585173453
Literal LRS Estimate: u = 2, v = 4, p-hat = 0.003907713861229295, p_u = 0.0041349846806774507
        T-Tuple Test Estimate = 7.814656 / 8 bit(s)
        LRS Test Estimate (bit string) = 0.998198 / 1 bit(s)
        LRS Test Estimate = 7.917902 / 8 bit(s)

Running Predictor Estimates...
Bitstring MultiMCW Prediction Estimate: N = 3999937, Pglobal' = 0.50086034082526742 (C = 2000834) Plocal can't affect result (r = 19)
        Multi Most Common in Window (MultiMCW) Prediction Test Estimate (bit string) = 0.997520 / 1 bit(s)                                                   Literal MultiMCW Prediction Estimate: N = 499937, Pglobal' = 0.0041831345534559658 (C = 1977) Plocal can't affect result (r = 3)
        Multi Most Common in Window (MultiMCW) Prediction Test Estimate = 7.901200 / 8 bit(s)
Bitstring Lag Prediction Estimate: N = 3999999, Pglobal' = 0.50108358234786721 (C = 2001758) Plocal can't affect result (r = 20)
        Lag Prediction Test Estimate (bit string) = 0.996877 / 1 bit(s)
Literal Lag Prediction Estimate: N = 499999, Pglobal' = 0.0041229395067461441 (C = 1948) Plocal can't affect result (r = 3)
        Lag Prediction Test Estimate = 7.922111 / 8 bit(s)
Bitstring MultiMMC Prediction Estimate: N = 3999998, Pglobal' = 0.5007807076116616 (C = 2000546) Plocal can't affect result (r = 19)
        Multi Markov Model with Counting (MultiMMC) Prediction Test Estimate (bit string) = 0.997749 / 1 bit(s)
Literal MultiMMC Prediction Estimate: N = 499998, Pglobal' = 0.004155874273437418 (C = 1964) Plocal can't affect result (r = 3)                                      Multi Markov Model with Counting (MultiMMC) Prediction Test Estimate = 7.910632 / 8 bit(s)                                                           Bitstring LZ78Y Prediction Estimate: N = 3999983, Pglobal' = 0.50072708411917954 (C = 2000324) Plocal can't affect result (r = 24)                                   LZ78Y Prediction Test Estimate (bit string) = 0.997904 / 1 bit(s)
                                  Literal LZ78Y Prediction Estimate: N = 499983, Pglobal' = 0.0041539410832216817 (C = 1963) Plocal can't affect result (r = 3)                                        LZ78Y Prediction Test Estimate = 7.911304 / 8 bit(s)

H_original: 7.814656
H_bitstring: 0.888637
min(H_original, 8 X H_bitstring): 7.109100
```

```plaintext
./ea_conditioning -v 368 256 256 327.0186
n_in = 368
n_out = 256
nw = 256
h_in = 327.0185999999999999776
Attempting to compute entropy with 736 bits of precision.
Output_Entropy(*) = 255.9999999999999996252
(Vetted) h_out = 255.9999999999999996252
epsilon = 2^(-59.23561681352755137891): FIPS 140-3 IG D.K Resolution 19 Full Entropy if the conditioning component security strength is >= 256
```

### **5. OTP Generation**

One-time passwords are generated by combining a hardware random number with a microsecond timestamp:

```plaintext
OTP = SHA-256(random_number_bytes || timestamp_bytes) mod 1,000,000
```

This produces a 6-digit code. The random number comes from TU Ankaja (not pseudo-random), and the timestamp adds uniqueness even if the same number appears twice.

### **6. Real-Time Dashboard (Next.js)**

Three pages, each serving a different purpose:

- **OTP Generator** - Generate hardware-based OTPs with one click. Shows source number, timestamp, and history.
- **Raw Data Viewer** - Live sensor graphs for all the sensor data, updating every 3 seconds.

### **7. MYOSA Libraries & Modules Used**

| MYOSA Module | Library / Interface | Purpose |
|---|---|---|
| MYOSA Motherboard (ESP32) | `WiFi.h`, `PubSubClient.h`, DAC (`dacWrite`), ADC (`analogRead`) | WiFi connectivity, MQTT publishing, MOSFET gate drive, noise sampling |
| MYOSA Accelerometer/Gyroscope | `Wire.h` (I2C, address `0x68`) | 6-axis motion data (accel x/y/z, gyro x/y/z) for entropy mixing |
| MYOSA Light/Proximity (APDS9960) | `LightProximityAndGesture.h`
 (I2C) | RGB color values and ambient light intensity |
| MYOSA OLED Display | `oled.h` (I2C) | On-device status display |
| PMS5003 Particle Sensor | UART (`Serial2`) | PM1.0, PM2.5, PM10 particle concentration readings |

> **Note:** The PMS5003 is external sensors not included in the standard MYOSA kit. This was added to increase the number of independent entropy sources.

---

## Usage Instructions

### Starting the Full Wireless Pipeline

1. Flash the ESP32 with the MQTT firmware:

```plaintext
cd firmware/mqtt
# Open entropy_mqtt.ino in Arduino IDE
# Update WIFI_SSID, WIFI_PASS, MQTT_SERVER
# Flash to ESP32
```

2. Start the Mosquitto MQTT broker:

```plaintext
mosquitto -d
```

3. Start the Rust backend:

```plaintext
./scripts/start-wireless.sh
```

Or manually with environment variables:

```plaintext
cd entropy-engine
ENTROPY_MODE=otp \
  MQTT_HOST=<your-broker-ip> \
  MQTT_PORT=1883 \
  MQTT_USER=<your-mqtt-username> \
  MQTT_PASS=<your-mqtt-password> \
  cargo run
```

4. Start the frontend dashboard:

```plaintext
cd frontend
npm install
npm run dev
```

5. Open `http://localhost:3000` in your browser.

### Running Without Hardware (Simulation)

If you do not have the ESP32 or MYOSA board, you can test with the simulator:

```plaintext
# Terminal 1: Start Mosquitto
mosquitto -d

# Terminal 2: Run the MQTT simulator
python3 scripts/mqtt_simulator.py

# Terminal 3: Start the engine in MQTT mode
cd entropy-engine
ENTROPY_MODE=mqtt cargo run

# Terminal 4: Start the dashboard
cd frontend && npm run dev
```

### Running the Firmware Simulator (No WiFi needed)

```plaintext
cd firmware
make run
```

This compiles and runs the C entropy simulator locally, printing a sample of generated entropy bytes.

---

## Tech Stack

* **ESP32 (MYOSA Motherboard)** - Microcontroller with built-in WiFi for wireless sensor data transmission
* **MYOSA Sensor Board** - 21-channel multi-sensor board (MOSFET noise, IMU, color, light, particle sensor)
* **IRF540N MOSFET** - N-channel MOSFET for analog noise generation (primary entropy source)
* **APDS9960** - RGB and ambient light sensor module
* **PMS5003** - Particle matter sensor for air quality readings
* **C** - Firmware for ADC noise reading and entropy mixing with avalanche function
* **Arduino (PubSubClient)** - MQTT client library for ESP32 WiFi publishing
* **Rust** - Backend with SHA-256 whitening, ChaCha20 DRBG, health monitoring, and HTTP API
* **rumqttc** - Rust MQTT client for subscribing to broker topics
* **axum** - Rust HTTP framework serving the REST API
* **Eclipse Mosquitto** - MQTT broker with username/password authentication
* **Next.js 16** - React framework for the dashboard frontend
* **Recharts** - Charting library for real-time sensor data visualization
* **Framer Motion** - Animation library for smooth UI transitions
* **Zustand** - Lightweight state management for React
* **Tailwind CSS v4** - Utility-first CSS framework

---

## Requirements / Installation

### Hardware Requirements

- MYOSA Motherboard (ESP32-based)
- MYOSA OLED Display
- MYOSA Accelerometer/Gyroscope Module
- MYOSA Light/Proximity Module (APDS9960)
- Particle Sensor (PMS5003)
- IRF540N n-channel MOSFET
- 2k ohm pull-up resistor, 82 ohm gate resistor
- Breadboard and jumper wires

### Software Requirements

- Rust toolchain (1.70+)
- Node.js (18+)
- Python 3 (for MQTT simulator)
- Arduino IDE (for flashing ESP32)

### Dependencies

Rust engine:

```plaintext
cd entropy-engine
cargo build
```

Frontend:

```plaintext
cd frontend
npm install
```

Python simulator:

```plaintext
pip install paho-mqtt
```

MQTT broker:

```plaintext
brew install mosquitto
```

### Quick Check (All Components)

```plaintext
./scripts/check-all.sh
```

This runs the firmware simulator, Rust tests (96 tests), and frontend type checking.

---

## File Structure

```
/tu-ankaja
  ├── assets/
  │   └── images/
  │       └── Tu-Ankaja/
  │           ├── mosfet-circuit-closeup.jpg
  │           ├── chaotic-box-inside-full.jpg
  │           ├── chaotic-box-sensors.jpg
  │           ├── chaotic-box-motor.jpg
  │           ├── dashboard-raw-data.png
  │           ├── dashboard-otp.png
  │           └── Nist.png
  │
  ├── firmware/
  │   ├── src/
  │   │   ├── main.c
  │   │   ├── adc/adc.c
  │   │   ├── sensors/sensors.c
  │   │   ├── entropy/entropy.c
  │   │   └── uart/uart.c
  │   ├── include/entropy_vault.h
  │   ├── mqtt/entropy_mqtt.ino
  │   └── Makefile
  │
  ├── entropy-engine/
  │   ├── src/
  │   │   ├── main.rs
  │   │   ├── lib.rs
  │   │   ├── api/mod.rs
  │   │   ├── whitening/mod.rs
  │   │   ├── pool/mod.rs
  │   │   ├── drbg/mod.rs
  │   │   ├── health/mod.rs
  │   │   ├── quality/mod.rs
  │   │   ├── security/mod.rs
  │   │   ├── crypto/mod.rs
  │   │   ├── mqtt/mod.rs
  │   │   ├── otp/mod.rs
  │   │   ├── otp_mqtt/mod.rs
  │   │   ├── server/mod.rs
  │   │   ├── parser/mod.rs
  │   │   ├── serial/mod.rs
  │   │   ├── models/mod.rs
  │   │   └── errors/mod.rs
  │   ├── tests/engine.rs
  │   └── Cargo.toml
  │
  ├── frontend/
  │   ├── app/
  │   │   ├── page.tsx
  │   │   ├── otp/page.tsx
  │   │   ├── data/page.tsx
  │   │   ├── entropy/page.tsx
  │   │   └── layout.tsx
  │   ├── components/
  │   ├── services/
  │   ├── store/
  │   ├── types/
  │   └── package.json
  │
  ├── scripts/
  │   ├── start-wireless.sh
  │   ├── mqtt_simulator.py
  │   ├── otp_simulator.py
  │   ├── check-all.sh
  │   ├── flash-esp32.sh
  │   └── start_all.sh
  │
  ├── myosa-demo.mp4
  └── LICENSE
```

---

## License

MIT License. See the LICENSE file for full text.

---

## Contribution Notes

This project is open source. If you want to contribute:

1. Fork the repository
2. Create a feature branch
3. Run `./scripts/check-all.sh` to make sure everything passes
4. Submit a pull request

Areas where contributions would be useful:

- Adding TLS/mTLS support for encrypted MQTT connections
- Adding more entropy health tests (e.g., NIST SP 800-22 test suite)
- Mobile app for monitoring the dashboard remotely
