# Humanoid Robot Project

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
