# IMU and Gyroscope Guide

This guide describes the MPU-6050 accelerometer/gyroscope integration currently implemented in the Arduino firmware and Rust GUI. Wiring is also listed in [HARDWARE_WIRING.md](HARDWARE_WIRING.md); packet fields are specified in [PROTOCOL.md](PROTOCOL.md).

## Sensor and wiring

The firmware expects an MPU-6050-compatible device at I2C address `0x68`. On an Arduino Mega 2560:

| MPU-6050 breakout | Mega 2560                                                 |
| ----------------- | --------------------------------------------------------- |
| SDA               | SDA / D20                                                 |
| SCL               | SCL / D21                                                 |
| GND               | GND                                                       |
| VCC               | Use the breakout's rated voltage; check its documentation |
| AD0               | LOW/GND, selecting address `0x68`                         |

The firmware initializes I2C at 400 kHz and checks the sensor's `WHO_AM_I` register for `0x68`. A bare MPU-6050 is not a 5V device; some breakout boards add a regulator or level shifting, others do not. Confirm the exact board before powering it.

## Measurement configuration

At startup, the firmware wakes the sensor, loads any saved offsets, and configures:

- Accelerometer range: ±2 g; scale factor 16384 raw counts per g.
- Gyroscope range: ±250 degrees per second; scale factor 131 raw counts per degree per second.
- Sample-rate divider: 9, giving a nominal sample rate of $1000/(1+9)=100$ Hz.
- Digital low-pass filter setting: 3, documented by the firmware as approximately 44 Hz bandwidth.

The loop reads 14 bytes of accelerometer, temperature, and gyroscope registers when the IMU is connected. Acceleration is converted to g, angular velocity to degrees per second, and temperature using the MPU-6050 formula `raw / 340 + 36.53` Celsius.

## What the application displays and transmits

The GUI's **Read IMU** action requests one sample; **Start stream** enables live samples at the configured rate (minimum 20 ms), and **Stop stream** stops telemetry. It displays acceleration in g, gyro rate in degrees per second, orientation in degrees, and temperature in Celsius. **Calibrate IMU** requests a stationary 1000-sample calibration.

The 54-byte `0xA0` response contains:

- Seven signed 16-bit big-endian values: ax, ay, az, gx, gy, gz, and temperature.
- Three IEEE-754 little-endian floats: ax_g, ay_g, and az_g.
- A 32-bit little-endian millisecond timestamp.
- Six IEEE-754 little-endian floats: gx_dps, gy_dps, gz_dps, roll, pitch, and yaw.

Rust accepts legacy 30-byte responses as well. Legacy data has no orientation values and its gyro rates are estimated from counts using the configured default range. The tilt monitor shows less than 15 degrees as level, 15-30 degrees as a warning, and 30 degrees or more as high tilt. These are advisory display thresholds only; they are not mechanical safety limits and do not trigger balance corrections.

## Calibration

Calibration is run from the GUI while the IMU is connected. Keep the sensor stationary and mounted in its intended neutral position during the sample interval. The firmware averages gyro offsets and accelerometer offsets, stores them in EEPROM, and loads them on the next startup.

The current offset calculation assumes gravity is approximately +1 g on the sensor's Z axis while stationary. If the board is mounted in a different orientation, this assumption can produce an incorrect Z offset. Mounting orientation and axis sign must be considered when interpreting readings. Calibration does not mechanically align servos or calibrate joint angles.

Calibration is initiated as request `0x41`; the firmware performs 1000 reads with approximately 2 ms delay between attempts. Saved offsets are retained in EEPROM. The GUI reports success only when it receives the command acknowledgement.

## Reading the values

- At rest on a level surface, the accelerometer magnitude should be close to 1 g, mostly on the axis aligned with gravity.
- A stationary gyroscope should read near zero after calibration. Before calibration, bias and drift are expected.
- The live GUI shows gyro rate in degrees per second. The wire packet also contains offset-corrected raw gyro counts for diagnostics.
- The accelerometer and gyroscope axes are sensor-local axes. Their relationship to robot left/right, forward/back, and up/down depends on how the module is mounted.
- Gyroscope integration drifts over time, especially yaw. The firmware combines gyro integration with accelerometer roll/pitch using a complementary filter; this is telemetry, not a validated balance-control loop.

## Troubleshooting

- **IMU unavailable:** check power, common ground, SDA/SCL, and that AD0 selects address `0x68`. The firmware marks the IMU unavailable if its identity read fails.
- **Values frozen or implausible:** verify the module is securely mounted, the robot is not vibrating during calibration, and the selected board is a compatible MPU-6050 breakout.
- **Gyro values seem stale or zero:** verify firmware 1.1.0 or later is flashed and IMU streaming is active.
- **Orientation fields show zero:** legacy 30-byte firmware frames do not carry orientation. Flash firmware 1.1.0 or later.
- **Calibration seems wrong:** place the sensor in its intended neutral orientation, keep it still throughout calibration, and note the firmware's +Z gravity assumption.

See [PROJECT_GUIDE.md](PROJECT_GUIDE.md) for build and operator workflow, and [PROTOCOL.md](PROTOCOL.md) for exact IMU packet byte offsets.
