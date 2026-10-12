<<<<<<< HEAD
# Serial Protocol Reference

This document describes the packet format shared by the Arduino Mega firmware and the Rust desktop application. The firmware definitions are in `arduino/HumanoidRobot/Config.h` and `arduino/HumanoidRobot/Protocol.cpp`; the desktop implementation is in `rust/src/serial.rs`.

## Transport

The protocol is carried over a byte-oriented UART connection. The default Bluetooth configuration uses Mega `Serial1` at 9600 baud, 8 data bits, no parity, and 1 stop bit (8-N-1). USB debug mode uses `Serial` at 115200 baud when `USE_BLUETOOTH_SERIAL` is set to `0`.

## Packet format

Every packet has this layout:

| Field               |       Size | Meaning                                   |
| ------------------- | ---------: | ----------------------------------------- |
| Start               |     1 byte | `0xAA`                                    |
| Command/response ID |     1 byte | Request or response type                  |
| Payload length      |     1 byte | Number of payload bytes, maximum 64       |
| Payload             | 0-64 bytes | Command-specific data                     |
| Checksum            |     1 byte | XOR of ID, length, and every payload byte |
| End                 |     1 byte | `0x55`                                    |

The total packet size is `payload length + 5` bytes. Multi-byte integer fields are big-endian unless an individual payload says otherwise. Packets are not escaped; the parser uses the length and ending byte to find frame boundaries.

## Requests

Servo indices are zero-based and use the mapping in `Config.h`: left hip, left knee, left ankle, right hip, right knee, right ankle, left shoulder, left elbow, left claw, right shoulder, right elbow, right claw, head pan, head tilt. Angle values use tenths of a degree (`angle_x10`); pulse values are microseconds.

|     ID | Request                  | Payload                                                                                                    |
| -----: | ------------------------ | ---------------------------------------------------------------------------------------------------------- |
| `0x01` | Ping                     | Empty                                                                                                      |
| `0x02` | Get firmware version     | Empty                                                                                                      |
| `0x10` | Set servo angle          | Servo index (u8), angle_x10 (u16)                                                                          |
| `0x11` | Set servo pulse          | Servo index (u8), pulse_us (u16)                                                                           |
| `0x12` | Get servo state          | Servo index (u8)                                                                                           |
| `0x13` | Set all servo angles     | Fourteen angle_x10 values (u16 each), in index order                                                       |
| `0x20` | Set servo limits         | Index (u8), min angle_x10 (u16), max angle_x10 (u16), min pulse (u16), max pulse (u16), center pulse (u16) |
| `0x21` | Get servo limits         | Servo index (u8)                                                                                           |
| `0x22` | Save robot configuration | Empty; stores servo configuration in EEPROM                                                                |
| `0x23` | Load robot configuration | Empty; loads servo configuration from EEPROM                                                               |
| `0x30` | Reset servos             | Empty; restore default servo settings and clear the emergency-stop latch                                   |
| `0x31` | Emergency stop           | Empty; hold current servo outputs and latch motion commands off                                            |
| `0x40` | Get IMU sample           | Empty                                                                                                      |
| `0x41` | Calibrate IMU            | Empty; firmware collects 1000 samples                                                                      |
| `0x42` | Set IMU offsets          | Six signed 16-bit offsets: ax, ay, az, gx, gy, gz                                                          |
| `0x43` | Get IMU offsets          | Empty                                                                                                      |
| `0x50` | Start stream             | Stream type (1 = IMU, 2 = servo states), then optional rate_ms (u16)                                       |
| `0x51` | Stop all streams         | Empty                                                                                                      |
| `0x52` | Set stream rate          | rate_ms (u16)                                                                                              |
| `0x60` | Run motion sequence      | Not implemented; firmware replies with invalid-command NACK                                                |
| `0x61` | Stop motion sequence     | Acknowledged, but no sequence runner is implemented                                                        |
| `0x70` | Get diagnostics          | Empty                                                                                                      |

A zero-based servo index from 0 through 13 is valid. The firmware's default angle range is 0-180 degrees and default pulse range is 500-2500 microseconds; physical joint limits vary and must be calibrated for the specific robot.

## Responses

|     ID | Response         | Payload                                                                                                    |
| -----: | ---------------- | ---------------------------------------------------------------------------------------------------------- |
| `0x80` | ACK              | Original request ID (u8)                                                                                   |
| `0x81` | NACK             | Original request ID (u8), error code (u8)                                                                  |
| `0x82` | Firmware version | Major (u8), minor (u8), patch (u8), servo count (u8), reserved (u8)                                        |
| `0x90` | Servo state      | Index (u8), angle_x10 (u16), pulse_us (u16), attached (u8)                                                 |
| `0x92` | Servo limits     | Index (u8), min angle_x10 (u16), max angle_x10 (u16), min pulse (u16), max pulse (u16), center pulse (u16) |
| `0xA0` | IMU sample       | 54-byte payload described below                                                                            |
| `0xA1` | IMU offsets      | Six signed 16-bit offsets, ax through gz                                                                   |
| `0xB0` | Diagnostics      | 60-byte payload described below                                                                            |
| `0xFF` | Firmware error   | Error code (u8)                                                                                            |

The firmware sends a version response at startup. A ping is considered successful only when an ACK containing request ID `0x01` is received. Opening a serial port alone does not verify that the robot is responding.

### IMU payload (`0xA0`)

The 54-byte sample contains seven signed 16-bit values in big-endian order (ax, ay, az, gx, gy, gz, temperature), followed by ax_g, ay_g, and az_g as four-byte IEEE-754 little-endian floats, a four-byte little-endian millisecond timestamp, then gyro rates gx_dps, gy_dps, gz_dps and orientation roll, pitch, yaw as six four-byte IEEE-754 little-endian floats. Rust remains compatible with legacy 30-byte samples; legacy packets derive gyro rates from raw counts and contain no transmitted orientation.

See [IMU_GYROSCOPE.md](IMU_GYROSCOPE.md) for sensor ranges, calibration behavior, axis interpretation, and host-display limitations.

### Diagnostics payload (`0xB0`)

| Byte offsets | Size | Value                                            | Encoding                 |
| ------------ | ---: | ------------------------------------------------ | ------------------------ |
| 0-3          |    4 | Uptime milliseconds                              | u32 big-endian           |
| 4-5          |    2 | Free RAM bytes                                   | u16 big-endian           |
| 6-33         |   28 | Estimated servo currents, 14 values              | u16 big-endian each      |
| 34-35        |    2 | Estimated total current mA                       | u16 big-endian           |
| 36-37        |    2 | Monitored voltage mV                             | u16 big-endian           |
| 38-39        |    2 | IMU temperature                                  | signed 16-bit big-endian |
| 40-43        |    4 | Received packet count                            | u32 big-endian           |
| 44-47        |    4 | Transmitted packet count                         | u32 big-endian           |
| 48-51        |    4 | CRC error count                                  | u32 big-endian           |
| 52-55        |    4 | Timeout error count                              | u32 big-endian           |
| 56           |    1 | Flags: bit 0 emergency stop, bit 1 IMU connected | Bit field                |
| 57-59        |    3 | Firmware major, minor, patch                     | u8 values                |

Current values are estimates based on servo positions, not measured current-sensor readings. Voltage is meaningful only when the optional divider is connected to analog input A0 and its ratio matches the firmware assumptions.

## Error codes

|   Code | Meaning                   |
| -----: | ------------------------- |
| `0x01` | Invalid command           |
| `0x02` | Invalid parameter         |
| `0x03` | Checksum failure          |
| `0x04` | Invalid servo index       |
| `0x05` | Angle out of range        |
| `0x06` | Pulse out of range        |
| `0x07` | IMU not found             |
| `0x08` | IMU initialization failed |
| `0x09` | EEPROM write failed       |
| `0x0A` | Safety violation          |
| `0x0B` | Communication timeout     |
| `0x0C` | Sequence already running  |

## Timing and safety

The firmware updates servos at 50 Hz and emits diagnostics every five seconds. After at least one valid command, a five-second interval with no further valid commands triggers the communication watchdog and latches an emergency stop. `0x31` holds the current servo output; `0x30` is the explicit reset-to-defaults and recovery command. The GUI exposes live IMU streaming; servo streaming remains available in firmware/API but is not currently shown as a live GUI control.

## Verification

Run the Rust protocol parser tests from `rust/` with `cargo test`. Tests cover full/truncated diagnostics payloads, 54-byte IMU telemetry, and legacy 30-byte IMU compatibility. Compile the firmware from the project root with:

```bash
arduino-cli compile --fqbn arduino:avr:mega arduino/HumanoidRobot
```
=======
# Serial Protocol Documentation

Complete specification of the serial communication protocol between the Rust desktop application and Arduino firmware.

## Overview

- **Baud Rate**: 115200
- **Data Bits**: 8
- **Parity**: None
- **Stop Bits**: 1
- **Line Ending**: Newline (`\n`, ASCII 10)
- **Timeout**: 100ms (Arduino side)

## Protocol Format

All communication is text-based, newline-terminated ASCII strings.

### Request Format

```
COMMAND [ARG1] [ARG2] ...
```

- Commands are case-sensitive (uppercase)
- Arguments are space-separated
- Each request ends with a newline character
- Maximum line length: 64 characters (Arduino buffer)

### Response Format

```
RESPONSE [DATA]
```

- Responses are case-sensitive
- Each response ends with a newline character
- Error responses start with "ERROR"

## Commands

### ENABLE

Enable all servos (attach to pins).

**Request**:
```
ENABLE
```

**Success Response**:
```
OK
```

**Description**:
- Attaches all servo objects to their pins
- Servos can now receive movement commands
- Servos remain at their current angles (home position by default)
- If already enabled, command is ignored

**Example**:
```
Request:  ENABLE
Response: OK
```

---

### DISABLE

Disable all servos (detach from pins).

**Request**:
```
DISABLE
```

**Success Response**:
```
OK
```

**Description**:
- Detaches all servo objects from their pins
- Servos relax and stop holding position
- Servos cannot receive movement commands while disabled
- If already disabled, command is ignored

**Example**:
```
Request:  DISABLE
Response: OK
```

---

### SET

Set a specific servo to an angle.

**Request**:
```
SET <index> <angle>
```

**Parameters**:
- `index`: Servo index (0-13)
- `angle`: Angle in degrees (varies by servo limits)

**Success Response**:
```
OK
```

**Error Response**:
```
ERROR angle out of bounds for <servo_name>: <angle> (min=<min>, max=<max>)
ERROR invalid servo index: <index>
```

**Description**:
- Sets the target angle for a specific servo
- Angle is validated against servo limits before applying
- If servos are disabled, angle is stored but not applied
- When servos are enabled, they move to stored angles
- Angle limits are conservative defaults; calibrate per your robot

**Servo Index Mapping**:
- 0: left hip
- 1: left knee
- 2: left ankle
- 3: right hip
- 4: right knee
- 5: right ankle
- 6: left shoulder
- 7: left elbow
- 8: left claw/wrist
- 9: right shoulder
- 10: right elbow
- 11: right claw/wrist
- 12: head pan
- 13: head tilt

**Angle Limits** (conservative defaults):
- Hip, knee, shoulder, elbow, claw: -90 to 90 degrees
- Ankle, head tilt: -45 to 45 degrees
- Head pan: -90 to 90 degrees

**Examples**:
```
Request:  SET 0 45
Response: OK

Request:  SET 0 100
Response: ERROR angle out of bounds for left hip: 100 (min=-90, max=90)

Request:  SET 14 0
Response: ERROR invalid servo index: 14
```

---

### GET

Get the current angle of a specific servo.

**Request**:
```
GET <index>
```

**Parameters**:
- `index`: Servo index (0-13)

**Success Response**:
```
GET <index> <angle>
```

**Error Response**:
```
ERROR invalid servo index: <index>
```

**Description**:
- Returns the current stored angle for a servo
- This is the last set angle, not necessarily the physical position
- Works whether servos are enabled or disabled

**Examples**:
```
Request:  GET 0
Response: GET 0 45

Request:  GET 14
Response: ERROR invalid servo index: 14
```

---

### HOME

Move all servos to their home position.

**Request**:
```
HOME
```

**Success Response**:
```
OK
```

**Description**:
- Sets all servos to their home angle (0 degrees by default)
- If servos are enabled, they move to home position
- If servos are disabled, angles are stored but not applied
- Useful for returning to a known safe state

**Example**:
```
Request:  HOME
Response: OK
```

---

### MPU

Read MPU-6050 sensor data.

**Request**:
```
MPU
```

**Success Response**:
```
MPU <accel_x> <accel_y> <accel_z> <gyro_x> <gyro_y> <gyro_z>
```

**Error/No Sensor Response**:
```
MPU 0 0 0 0 0 0
```

**Description**:
- Reads current accelerometer and gyroscope data from MPU-6050
- Values are raw 16-bit integers from the sensor
- Accelerometer: typically in range -16384 to 16384 (±2g range)
- Gyroscope: typically in range -32768 to 32767 (±250 deg/s range)
- If MPU-6050 is not detected or not initialized, returns all zeros
- Units depend on sensor configuration (not calibrated in firmware)

**Examples**:
```
Request:  MPU
Response: MPU 1200 -300 16000 100 -50 200

Request:  MPU
Response: MPU 0 0 0 0 0 0  (sensor not detected)
```

---

### STATUS

Get robot status information.

**Request**:
```
STATUS
```

**Success Response**:
```
STATUS enabled=<true|false> mpu=<true|false> servos=<angle0>,<angle1>,...,<angle13>
```

**Description**:
- Returns current state of the robot
- `enabled`: whether servos are attached (true/false)
- `mpu`: whether MPU-6050 is initialized (true/false)
- `servos`: comma-separated list of all 14 servo angles

**Example**:
```
Request:  STATUS
Response: STATUS enabled=true mpu=true servos=0,0,0,0,0,0,45,0,0,0,0,0,0,0
```

---

### PING

Ping the Arduino to check connectivity.

**Request**:
```
PING
```

**Success Response**:
```
PONG
```

**Description**:
- Simple connectivity test
- Arduino responds immediately with PONG
- Useful for testing serial connection
- Can be used to check if Arduino is responsive

**Example**:
```
Request:  PING
Response: PONG
```

---

## Error Responses

All error responses follow this format:

```
ERROR <error_message>
```

### Common Errors

| Error Message | Cause |
|---------------|-------|
| `unknown command: <cmd>` | Invalid command sent |
| `invalid SET syntax` | SET command missing arguments |
| `invalid servo index: <n>` | Servo index out of range (0-13) |
| `angle out of bounds for <name>: <angle> (min=<min>, max=<max>)` | Angle exceeds servo limits |

## Startup Sequence

### Arduino Startup

1. Arduino powers on or resets
2. Serial port initializes at 115200 baud
3. I2C initializes for MPU-6050
4. MPU-6050 initialization attempted
5. Servo objects created but NOT attached (disabled)
6. All servo angles set to home position (0 degrees)
7. Arduino sends: `READY`
8. Arduino enters command processing loop

### Desktop Application Startup

1. Open serial port at 115200 baud
2. Wait for `READY` message from Arduino (timeout: 5 seconds)
3. If `READY` received, connection is established
4. Application can now send commands

**Example startup sequence**:
```
Arduino: READY
Desktop: PING
Arduino: PONG
Desktop: STATUS
Arduino: STATUS enabled=false mpu=true servos=0,0,0,0,0,0,0,0,0,0,0,0,0,0
```

## Implementation Notes

### Arduino Side

- Uses `Serial.readStringUntil('\n')` for command parsing
- 100ms timeout on serial reads
- Commands processed in main loop
- Servo updates every 20ms (50Hz)
- MPU-6050 polled on demand (not continuous)

### Rust Side

- Uses `serialport` crate for communication
- 100ms timeout on serial reads
- Commands sent with newline termination
- Responses parsed line-by-line
- Angle validation performed before sending SET command

### Synchronization

- Commands are synchronous (request-response)
- No pipelining of commands
- Each command must complete before next is sent
- Desktop waits for response before proceeding

## Protocol Version

Current version: 1.0

This protocol is intentionally simple:
- Text-based for easy debugging
- No binary encoding
- No compression
- No encryption
- No authentication

Future versions may add:
- Binary protocol for efficiency
- Batch commands for multiple servos
- Streaming sensor data
- Configuration commands

## Testing the Protocol

### Manual Testing (Using Serial Terminal)

1. Open serial terminal (e.g., `screen`, `minicom`, Arduino Serial Monitor)
2. Connect to Arduino at 115200 baud
3. Wait for `READY` message
4. Send commands manually:
   ```
   PING
   STATUS
   ENABLE
   SET 0 45
   GET 0
   HOME
   DISABLE
   ```

### Automated Testing (Using Rust Application)

```bash
# Test connectivity
cargo run -- /dev/ttyACM0 ping

# Test status
cargo run -- /dev/ttyACM0 status

# Test servo control
cargo run -- /dev/ttyACM0 enable
cargo run -- /dev/ttyACM0 set 0 45
cargo run -- /dev/ttyACM0 get 0
cargo run -- /dev/ttyACM0 home
cargo run -- /dev/ttyACM0 disable

# Test MPU
cargo run -- /dev/ttyACM0 mpu
```

### Unit Tests

Rust code includes unit tests for protocol parsing:

```bash
cargo test
```

Tests cover:
- Command serialization
- Response parsing
- Error handling
- Edge cases

## Troubleshooting Protocol Issues

### No Response

**Check**:
- Baud rate matches (115200)
- Serial port is correct
- Arduino is powered
- USB cable is data cable (not charge-only)
- Wait for `READY` message before sending commands

### Garbled Response

**Check**:
- Baud rate matches
- No noise on serial line
- USB cable quality
- Ground connection between Arduino and computer

### Timeout

**Check**:
- Arduino is responsive (LED blinks)
- Serial port not in use by another program
- No hardware conflicts
- Arduino code is running (not stuck)

### Wrong Response Format

**Check**:
- Protocol version matches
- Arduino firmware is up to date
- Rust application is up to date
- No extra characters in command

## Protocol Compatibility

This protocol is designed to be:
- **Simple**: Easy to implement and debug
- **Extensible**: New commands can be added
- **Backward compatible**: Old commands remain valid
- **Cross-platform**: Works on any system with serial support

## Future Enhancements

Potential future protocol additions:
- `BATCH` command for setting multiple servos at once
- `STREAM` command for continuous sensor data
- `CONFIG` command for changing servo limits
- `CALIBRATE` command for automatic limit detection
- `SAVE`/`LOAD` command for storing positions
- Binary protocol option for efficiency

These are not currently implemented and should not be assumed to exist.
>>>>>>> origin/main
