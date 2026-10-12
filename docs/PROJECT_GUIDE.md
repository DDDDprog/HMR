# Project Guide

This guide summarizes what the current source implements, how the Arduino and Rust programs fit together, and how to build and operate them. Hardware pin and power details are in [HARDWARE_WIRING.md](HARDWARE_WIRING.md), sensor behavior is in [IMU_GYROSCOPE.md](IMU_GYROSCOPE.md), Bluetooth host setup is in [BLUETOOTH_SETUP.md](BLUETOOTH_SETUP.md), and packet bytes are in [PROTOCOL.md](PROTOCOL.md).

## Components

The project has two runtime components:

- **Arduino Mega 2560 firmware** reads commands, controls 14 servos, samples an optional MPU-6050, and sends acknowledgements and telemetry.
- **Rust desktop GUI** opens a serial port, verifies the robot with a ping, and provides manual controls and status displays.

Normal operation can use Bluetooth SPP through an HC-05/HC-06 UART module. Firmware upload uses the Mega USB bootloader; USB is not the required runtime control link once the firmware has been flashed.

## Source map

### Arduino firmware: `arduino/HumanoidRobot/`

- `HumanoidRobot.ino` initializes UART, servos, and IMU; processes packets; dispatches commands; and sends diagnostics/streams.
- `Config.h` defines the 14 servo indices/pins, UART selection and baud rates, protocol IDs, and safety constants.
- `ServoController.h/.cpp` attaches servos, converts angles and pulses, enforces configured limits, stores servo configuration in EEPROM, and implements emergency stop/reset.
- `IMUHandler.h/.cpp` communicates with an MPU-6050 over I2C, converts raw measurements, computes orientation internally, and stores calibration offsets in EEPROM.
- `Protocol.h/.cpp` frames packets and serializes responses, including IMU and diagnostics payloads.

### Rust GUI: `rust/`

- `src/main.rs` loads configuration, initializes logging and the serial manager, and launches eframe.
- `src/config.rs` defines defaults and reads/writes the RON application config.
- `src/gui.rs` provides port selection, connect/ping status, servo angle/pulse and limits, IMU/calibration, diagnostics, EEPROM actions, and safety controls.
- `src/serial.rs` encodes requests, validates/parses responses, and owns serial communication. The parser regression tests are at the end of this file.
- `src/utils.rs` is currently an unused helper placeholder.

## Build and test

Run Rust checks from the `rust` directory:

```bash
cargo fmt --check
cargo check
cargo test
cargo run
```

Compile firmware from the project root without uploading:

```bash
arduino-cli compile --fqbn arduino:avr:mega arduino/HumanoidRobot
```

Upload through a connected Mega USB bootloader. Check the actual port first:

```bash
arduino-cli board list
arduino-cli compile --fqbn arduino:avr:mega --upload -p /dev/ttyACM0 arduino/HumanoidRobot
```

Replace `/dev/ttyACM0` with the detected board port. The Rust suite has seven tests: four parser tests for diagnostics and current/legacy IMU payloads, plus three transport-config compatibility/default tests. The `tests/` directory has no standalone test files.

## Runtime setup

The GUI provides **Wired USB** and **Wireless Bluetooth** profiles and connects manually by default. Wired USB is the default at 115200 baud (`/dev/ttyACM0` on Linux); wireless is optional at 9600 baud (`/dev/rfcomm0` on Linux, `COM3` on Windows). The GUI allows port and baud editing while disconnected and saves app settings under the platform config directory at `humanoid-robot-gui/config.ron`.

Firmware now defaults to `USE_BLUETOOTH_SERIAL 0`, selecting USB `Serial` at 115200 baud for runtime control. Set it to `1` to opt into Mega `Serial1` at 9600 baud. Select the matching GUI profile; switching profiles does not rebuild firmware. Firmware upload remains over USB in either mode.

The GUI reports **Serial port: OPEN/CLOSED** separately from **Robot: RESPONDING/NO RESPONSE**. The first is host device state; the second requires a valid ping acknowledgement from the firmware. It sends a ping heartbeat every second while responsive to service the firmware's five-second watchdog. Disconnect before changing wired/wireless profile.

## Operator controls

- **Joint groups:** filter control to Legs, Arms, Hands, Head, or All, then select an individual joint. **Apply angle / Apply pulse** sends that joint's target; the firmware interpolates at 50 Hz and validates its configured limits. All-servo protocol commands validate the complete target set before applying any target.
- **Read state:** request the selected joint's current firmware-reported angle, pulse, and attached state.
- **Read joint limits / Apply joint limits:** read or update the selected joint's angle and pulse ranges. Changes are volatile until **Save robot config** writes them to EEPROM.
- **Read IMU / Calibrate IMU:** sample the MPU-6050 or collect calibration offsets. Keep the robot still during calibration; the firmware collects 1000 samples.
- **Read diagnostics:** request uptime, free RAM, estimated servo currents, optional A0 voltage, IMU/stop flags, and packet/error counters. Current values are estimates, not current-sensor measurements.
- **Save robot config / Load robot config:** persist or restore servo settings from Mega EEPROM. After loading, read joint limits to refresh the GUI's displayed values.
- **Save app settings:** persist host port, baud, and other app configuration locally.
- **Emergency Stop:** latch motion commands off while maintaining the current servo pulse outputs. It does not cut electrical power or act as a mechanical brake.
- **Reset servos:** return servos to configured defaults and clear the emergency-stop latch.

The firmware watchdog triggers an emergency stop after five seconds without a valid command, once at least one valid packet has been received. The GUI's one-second heartbeat prevents idle connected sessions from tripping it; communication loss still causes a stop. Use appropriate power switching for a physical emergency stop.

## Known limitations

- Camera settings are defined in the Rust config model, but no camera capture or video display is implemented.
- Person detection and automatic head tracking are not implemented. They need a desktop vision runtime such as OpenCV; this environment currently has camera devices but no OpenCV runtime. Head rotation must remain user-commanded until detection, calibration, and bounded tracking are tested with the actual robot.
- Motion sequence execution is not implemented; `CMD_RUN_SEQUENCE` is rejected by the firmware.
- The GUI supports live IMU streaming and an advisory tilt monitor. The monitor does not automatically move joints to balance the robot.
- The displayed servo-current values are estimates. Accurate voltage monitoring requires a correctly designed divider on A0.
- Actual robot response, servo calibration, and Bluetooth operation require the corresponding hardware and cannot be proven by compiling alone.
