# Humanoid Robot Project

<<<<<<< HEAD
This repository contains a humanoid robot control stack with:

- Arduino firmware for the robot controller
- A Rust desktop GUI for control and monitoring
- Serial protocol and calibration logic
- Bluetooth-ready UART communication for wireless control
- A shared binary serial protocol documented in [docs/PROTOCOL.md](docs/PROTOCOL.md)

## Project layout

- `arduino/HumanoidRobot/` — Arduino Mega firmware
- `rust/` — desktop control GUI
- `docs/` — project reference material
- `docs/PROJECT_GUIDE.md` — source map, setup, runtime, and limitations
- `docs/HARDWARE_WIRING.md` — 14-servo signal pins, power, Bluetooth, and IMU wiring
- `docs/IMU_GYROSCOPE.md` — MPU-6050 measurements, calibration, units, and GUI behavior
- `docs/BLUETOOTH_SETUP.md` — Bluetooth wiring and host setup
- `docs/PROTOCOL.md` — serial packet and payload reference

## Quick start

### 1. Arduino firmware

Connect an Arduino Mega 2560 over USB for firmware upload, then open the sketch in:

- `arduino/HumanoidRobot/HumanoidRobot.ino`

Then compile and upload with the Arduino IDE or `arduino-cli`.

To compile without uploading:

```bash
arduino-cli compile --fqbn arduino:avr:mega arduino/HumanoidRobot
```

To compile and upload from Linux, first check the detected port with `arduino-cli board list`, then use the reported Mega port, for example:

```bash
arduino-cli compile --fqbn arduino:avr:mega --upload -p /dev/ttyACM0 arduino/HumanoidRobot
```

Uploading normally uses the Mega's USB bootloader. Bluetooth is the runtime control link after firmware has been flashed; do not disconnect USB until the upload finishes.

### 2. Rust GUI

From the `rust` folder, check and test the application before running it:

```bash
cargo check
cargo test
cargo run
```

The GUI connects only after you select a link and press **Connect**. It reports serial-port state separately from robot ping response. **OPEN** only means the host opened the selected serial device; **RESPONDING** means the Mega returned a valid protocol ping ACK.

## Bluetooth support

This project now supports wireless serial communication using a Bluetooth UART module such as HC-05 or HC-06.

### Hardware wiring (Arduino Mega 2560)

The complete 14-servo signal-pin table and safe power instructions are in [docs/HARDWARE_WIRING.md](docs/HARDWARE_WIRING.md).

Recommended UART wiring:

- Bluetooth TX → Arduino `19` (`RX1`)
- Bluetooth RX → Arduino `18` (`TX1`) through a 5V-to-3.3V level shifter/divider
- VCC → 5V
- GND → GND

Use a separate supply sized for the servos; do not power the servo bank from the Mega's 5V pin. Join the servo supply, Mega, and Bluetooth module grounds. Verify the Bluetooth board's supply requirements; bare HC-05 radio modules are 3.3V devices, while some breakout boards include a 5V regulator. Mega TX1 is 5V logic, so protect the module RX input with a level shifter or divider.

Bluetooth is optional. The firmware defaults to wired USB; set `USE_BLUETOOTH_SERIAL` to `1` in `arduino/HumanoidRobot/Config.h` to opt into Bluetooth runtime control.

### Bluetooth pairing

Follow [docs/BLUETOOTH_SETUP.md](docs/BLUETOOTH_SETUP.md) to pair the module and create or select the host serial port. Pairing alone may not create `/dev/rfcomm0`.

The GUI has **Wired USB** and **Wireless Bluetooth** link modes. Select the mode matching the firmware build: wired uses USB serial at 115200 baud with `USE_BLUETOOTH_SERIAL 0`; wireless uses Bluetooth SPP at 9600 baud with `USE_BLUETOOTH_SERIAL 1`. Disconnect before changing modes; switching in the GUI does not rebuild firmware. The GUI verifies the robot with a ping; an open serial port by itself does not mean the Mega is responding.

### Firmware setting

To opt into wireless runtime control, set `arduino/HumanoidRobot/Config.h`:

```cpp
#define USE_BLUETOOTH_SERIAL 1
```

With that enabled, the robot uses `Serial1` instead of USB serial, allowing wireless control at 9600 baud. Recompile and upload after changing the flag.

For USB debugging or direct tethering, set this to `0`.

## Serial port defaults

The GUI defaults to wired USB:

- Linux: `/dev/ttyACM0`
- Windows/macOS: select the detected USB serial port

Wireless remains optional:

- Linux: `/dev/rfcomm0`
- Windows: select the Bluetooth SPP COM port (often `COM3`)

The timeout defaults to 1000 ms and auto-connect is disabled. The selected link profile sets its baud default. Edit the port and baud rate in the GUI while disconnected, then use **Save app settings**. The saved RON file is in the platform configuration directory under `humanoid-robot-gui/config.ron` (typically `~/.config/humanoid-robot-gui/config.ron` on Linux).

If the saved configuration cannot be parsed, the app logs the error and starts with defaults.

## Protocol note

The Arduino and Rust app share a custom packet protocol defined by the command and response IDs in:

- `arduino/HumanoidRobot/Config.h`
- `arduino/HumanoidRobot/Protocol.h`
- `rust/src/serial.rs`

See [docs/PROTOCOL.md](docs/PROTOCOL.md) for packet fields, payload layouts, units, error codes, and implementation limits. The protocol handles:

- servo commands
- IMU reads
- diagnostics
- streaming
- configuration save/load

The Rust parser regression tests cover complete/truncated diagnostics plus current and legacy IMU payloads. There are currently no separate test files under `tests/`.

## Troubleshooting

### The robot does not respond

- Check that the Mega and Bluetooth module are powered and that the Bluetooth device is paired and reachable.
- On Linux, confirm `/dev/rfcomm0` is bound to the module's Serial Port Profile; a local device node alone does not prove the radio link is active.
- Make sure the baud rate matches the module configuration.
- Confirm `USE_BLUETOOTH_SERIAL` is `1`, firmware uses `Serial1`, and the GUI baud rate matches `ROBOT_BAUD_RATE`.
- Check crossed UART wiring: module TX to Mega RX1 (pin 19), module RX to Mega TX1 (pin 18 through level shifting), and common ground.
- If **Serial port** says **OPEN** but **Robot** says **NO RESPONSE**, the host opened a device but did not receive a valid ping ACK. Check module pairing/RFCOMM state, robot power, wiring, and firmware baud.

### The GUI cannot connect

- Verify the serial port value in the app config.
- Ensure no other app is already holding the Bluetooth COM port.
- Try reconnecting after power cycling the robot.
- Review the detailed status message shown by the GUI and refresh the detected-port list.

## Desktop controls

The GUI currently provides:

- connection and robot ping status
- grouped leg, arm, hand, head, and all-joint selection with per-joint angle commands and state reads
- per-joint angle/pulse limit editing and reads
- on-demand and live IMU telemetry, calibration, and advisory tilt warnings
- diagnostics, firmware version, EEPROM save/load, and local settings save
- emergency stop and reset-to-defaults controls

The Emergency Stop command holds the current servo outputs and latches normal motion commands off. **Reset servos** restores defaults and clears the latch. The GUI sends a ping heartbeat every second while the robot is responding, servicing the firmware's five-second watchdog during idle periods. The watchdog still stops the robot if the app/link loses communication.

The IMU tilt monitor is advisory only; automatic balance control is not implemented and requires robot-specific mechanical calibration and safety validation. Camera capture and person detection/head-following are not implemented; this build has no vision runtime (OpenCV is not installed). Motion sequences are not implemented; the firmware rejects run-sequence commands.

## License

MIT / Apache-2.0
=======
A desktop-controlled humanoid robot using Arduino Mega 2560, 14 hobby servos, and MPU-6050 IMU sensor.

## Hardware

- **Microcontroller**: Arduino Mega 2560
- **Servos**: 14 hobby servos (standard PWM servos)
- **IMU**: GY-521 MPU-6050 (3-axis accelerometer + gyroscope)
- **Computer**: Kali Linux laptop (or any Linux system)
- **Camera**: USB camera (connected directly to laptop)
- **Power**: External regulated power supply for servos (required)

## Software

- **Arduino Firmware**: C++ firmware for servo control and IMU reading
- **Desktop Application**: Rust application for serial communication and control

## Quick Start

### 1. Install Arduino IDE

Download and install the Arduino IDE from [arduino.cc](https://www.arduino.cc/en/software).

### 2. Install Rust

If you don't have Rust installed, run:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 3. Install Required Rust Dependencies

```bash
cd /home/kali/index/ROBOT
cargo build
```

This will automatically install the required dependencies:
- `serialport` - Serial port communication
- `anyhow` - Error handling
- `thiserror` - Error types

### 4. Wire the Hardware

**IMPORTANT**: Read the wiring documentation before connecting anything:

- See [docs/PIN_MAP.md](docs/PIN_MAP.md) for complete pin assignments
- See [docs/WIRING.md](docs/WIRING.md) for detailed wiring instructions
- See [docs/POWER_BUDGET.md](docs/POWER_BUDGET.md) for power supply requirements
- See [docs/SAFETY.md](docs/SAFETY.md) for safety procedures

**Critical Safety Notes**:
- NEVER power servos from the Arduino 5V pin
- Use an external regulated power supply for servos
- Connect external power ground to Arduino ground
- Keep USB serial pins D0/D1 reserved for programming
- Servos start DISABLED - they won't move until you send ENABLE command

### 5. Upload Arduino Firmware

1. Open `arduino/humanoid_robot.ino` in the Arduino IDE
2. Select "Arduino Mega 2560" from Tools > Board
3. Select the correct USB port from Tools > Port (e.g., `/dev/ttyACM0`)
4. Click the Upload button (right arrow)

### 6. Connect to Arduino

Find your Arduino's serial port:

```bash
ls /dev/ttyACM* /dev/ttyUSB*
```

Common ports on Linux:
- `/dev/ttyACM0` or `/dev/ttyACM1` (USB)
- `/dev/ttyUSB0` or `/dev/ttyUSB1` (USB-serial adapters)

### 7. Run the Desktop Application

**Check status**:
```bash
cargo run -- /dev/ttyACM0 status
```

**Interactive mode** (recommended for first use):
```bash
cargo run -- /dev/ttyACM0 interactive
```

In interactive mode, type `help` to see available commands.

## Usage Examples

### Enable Servos
```bash
cargo run -- /dev/ttyACM0 enable
```

### Set a Servo Angle
```bash
# Set servo 0 (left hip) to 45 degrees
cargo run -- /dev/ttyACM0 set 0 45
```

### Get Current Servo Angle
```bash
cargo run -- /dev/ttyACM0 get 0
```

### Move All Servos to Home Position
```bash
cargo run -- /dev/ttyACM0 home
```

### Read IMU Data
```bash
cargo run -- /dev/ttyACM0 mpu
```

### Disable Servos
```bash
cargo run -- /dev/ttyACM0 disable
```

## Servo Index Mapping

| Index | Name           | Pin | Min Angle | Max Angle | Home |
|-------|----------------|-----|-----------|-----------|------|
| 0     | left hip       | D2  | -90       | 90        | 0    |
| 1     | left knee      | D3  | -90       | 90        | 0    |
| 2     | left ankle     | D4  | -45       | 45        | 0    |
| 3     | right hip      | D5  | -90       | 90        | 0    |
| 4     | right knee     | D6  | -90       | 90        | 0    |
| 5     | right ankle    | D7  | -45       | 45        | 0    |
| 6     | left shoulder  | D8  | -90       | 90        | 0    |
| 7     | left elbow     | D9  | -90       | 90        | 0    |
| 8     | left claw      | D10 | -90       | 90        | 0    |
| 9     | right shoulder | D11 | -90       | 90        | 0    |
| 10    | right elbow    | D12 | -90       | 90        | 0    |
| 11    | right claw     | D13 | -90       | 90        | 0    |
| 12    | head pan       | D22 | -90       | 90        | 0    |
| 13    | head tilt      | D23 | -45       | 45        | 0    |

**Note**: These are conservative initial limits. You MUST calibrate them to your actual mechanical limits before moving servos.

## MPU-6050 IMU Connections

| MPU Pin | Arduino Pin | Function |
|---------|-------------|----------|
| VCC     | 5V          | Power (if board supports 5V) |
| GND     | GND         | Ground |
| SDA     | D20 (SDA)   | I2C Data |
| SCL     | D21 (SCL)   | I2C Clock |
| INT     | (unused)    | Interrupt (optional) |

## Serial Protocol

The desktop application communicates with the Arduino over USB serial at 115200 baud.

See [docs/PROTOCOL.md](docs/PROTOCOL.md) for complete protocol documentation.

## Testing

Run the Rust tests:

```bash
cargo test
```

Tests include:
- Serial protocol parsing
- Servo angle limit validation
- Command/response parsing

## Project Structure

```
ROBOT/
├── arduino/
│   └── humanoid_robot.ino    # Arduino firmware
├── src/
│   ├── main.rs                # Desktop application
│   ├── protocol.rs            # Serial protocol implementation
│   └── limits.rs              # Servo angle limits and validation
├── docs/
│   ├── PIN_MAP.md             # Pin assignments
│   ├── WIRING.md              # Wiring instructions
│   ├── POWER_BUDGET.md        # Power supply requirements
│   ├── PROTOCOL.md            # Serial protocol documentation
│   └── SAFETY.md              # Safety procedures
├── Cargo.toml                 # Rust project configuration
└── README.md                  # This file
```

## Important Safety Warnings

1. **External Power Required**: Never power servos from Arduino 5V pin
2. **Common Ground**: Connect external power ground to Arduino ground
3. **Calibration First**: Calibrate servo limits before full movement
4. **Mechanical Support**: Support the robot during initial tests
5. **Emergency Disconnect**: Keep power switch accessible for quick shutdown
6. **No Auto-Movement**: Servos are disabled on startup for safety

See [docs/SAFETY.md](docs/SAFETY.md) for detailed safety procedures.

## Troubleshooting

### Arduino not detected
- Check USB cable (use a data cable, not charge-only)
- Try different USB port
- Check `ls /dev/ttyACM*` and `ls /dev/ttyUSB*`
- Ensure Arduino is powered

### Permission denied on serial port
```bash
sudo usermod -a -G dialout $USER
# Log out and back in
```

Or use sudo temporarily:
```bash
sudo cargo run -- /dev/ttyACM0 status
```

### Servos not moving
- Ensure external power is connected
- Check if servos are enabled (send `enable` command)
- Verify servo signal wires are connected correctly
- Check power supply is adequate (see POWER_BUDGET.md)

### MPU-6050 not detected
- Check I2C connections (SDA=D20, SCL=D21)
- Verify MPU-6050 VCC connection (5V or 3.3V depending on board)
- Check ground connection
- Some GY-521 boards require 3.3V, not 5V

## Limitations

- No automatic balancing or walking (requires additional development)
- Conservative servo limits must be calibrated per robot
- Camera integration not yet implemented
- No inverse kinematics (manual servo control only)

## License

This project is provided as-is for educational purposes.

## Contributing

This is a personal project. Contributions welcome but please:
- Follow the existing code style
- Update documentation for any changes
- Test thoroughly with real hardware
- Respect safety requirements
>>>>>>> origin/main
