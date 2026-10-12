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
