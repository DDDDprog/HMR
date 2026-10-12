# Hardware Wiring and Servo Connections

This guide matches the pin assignments in `arduino/HumanoidRobot/Config.h` and `ServoController.cpp`. It covers signal wiring only where the firmware defines a pin. Servo power wiring must be designed for the actual servo models installed on the robot.

## Servo signal pin map

Each servo's signal lead connects to the Mega digital pin in this table. The index is zero-based and is also the index used by the GUI and serial protocol.

| Index | Joint          | Mega signal pin |
| ----: | -------------- | --------------: |
|     0 | Left Hip       |              D2 |
|     1 | Left Knee      |              D3 |
|     2 | Left Ankle     |              D4 |
|     3 | Right Hip      |              D5 |
|     4 | Right Knee     |              D6 |
|     5 | Right Ankle    |              D7 |
|     6 | Left Shoulder  |              D8 |
|     7 | Left Elbow     |              D9 |
|     8 | Left Claw      |             D10 |
|     9 | Right Shoulder |             D11 |
|    10 | Right Elbow    |             D12 |
|    11 | Right Claw     |             D13 |
|    12 | Head Pan       |             D22 |
|    13 | Head Tilt      |             D23 |

These are signal outputs controlled by the Arduino Servo library. The firmware does not provide servo power from these pins. Dedicated hardware PWM pins are not required by the Servo library for these outputs.

## Power and ground

A typical three-wire hobby servo has signal, positive supply, and ground leads. Wire the signal lead to the mapped Mega pin; connect the positive and ground leads to a regulated servo supply that meets the servo manufacturer's voltage and current requirements. Wire colors are not universal, so verify the servo datasheet before connecting power.

- Use a separate, adequately rated supply and distribution board for the servo bank. Size it for the combined worst-case/stall current of the servos that can move together.
- Do not power the servo bank from the Mega 5V pin, USB port, or an undersized breadboard rail.
- Connect the external servo supply ground to Mega GND so the PWM signal has a common reference.
- Do not connect the external servo positive rail to the Mega 5V rail unless the complete power design explicitly supports that connection. Avoid back-feeding power through USB.
- Add suitable branch protection and secure wiring against movement. Keep high-current servo wiring away from sensitive signal wiring where practical.
- Power the Mega from an appropriate regulated source and power the servos from their designed supply. Confirm polarity before connecting the battery.

The firmware defaults are 500-2500 microseconds and 0-180 degrees, but these are not guaranteed safe mechanical limits for a particular joint. Start with the robot mechanically supported, linkages unloaded where possible, and conservative limits calibrated for each servo. Keep clear of pinch points. At boot, the firmware attaches servos and commands their configured center pulse.

## Bluetooth UART module

Bluetooth control uses the Mega hardware UART `Serial1` when `USE_BLUETOOTH_SERIAL` is `1`.

| Bluetooth module | Arduino Mega 2560                                        |
| ---------------- | -------------------------------------------------------- |
| TX               | D19 / RX1                                                |
| RX               | D18 / TX1, through a 5V-to-3.3V level shifter or divider |
| GND              | GND shared with Mega and servo supply                    |
| VCC              | Module breakout's rated input; check its documentation   |

The Mega TX1 output is 5V logic. Protect the Bluetooth module RX input. Some HC-05/HC-06 breakout boards accept 5V VCC because they include a regulator; the bare radio module is 3.3V-only. Pairing and RFCOMM setup are in [BLUETOOTH_SETUP.md](BLUETOOTH_SETUP.md).

Firmware upload still uses the Mega USB bootloader. USB data can be disconnected for wireless runtime after the firmware has been flashed; the Mega and robot must remain powered.

## MPU-6050 IMU

The firmware uses I2C at address `0x68` and enables a 400 kHz bus. On a Mega 2560 the I2C pins are SDA/D20 and SCL/D21.

| MPU-6050 breakout | Arduino Mega 2560                                      |
| ----------------- | ------------------------------------------------------ |
| SDA               | SDA / D20                                              |
| SCL               | SCL / D21                                              |
| GND               | GND                                                    |
| VCC               | Breakout-rated voltage; verify the board specification |
| AD0               | GND/LOW for the firmware's expected address `0x68`     |

Do not assume a bare MPU-6050 accepts 5V. Some breakout boards include regulation and level shifting; check the exact board. The firmware reports the IMU unavailable if it cannot read the expected `WHO_AM_I` value at address `0x68`.

For sensor ranges, calibration assumptions, gyro units, and the limits of the current GUI telemetry, see [IMU_GYROSCOPE.md](IMU_GYROSCOPE.md).

## Optional voltage monitor

Analog input A0 is used for optional battery-voltage diagnostics. The code assumes a 100 kOhm / 10 kOhm divider (approximately 11:1) and a 5V ADC reference. Never connect a battery directly to A0; keep the voltage at the Mega input within its electrical limits. Choose and verify the divider for the battery's maximum voltage. The current diagnostic is only as accurate as the divider and reference, and servo-current readings are software estimates rather than sensor measurements.

## Safe first-power checklist

1. Keep the robot supported and clear of people and obstructions; initially disconnect servo power while checking signal/power wiring.
2. Verify every servo model's wire order, rated voltage, polarity, and current requirements.
3. Verify the Mega pins against the table and connect all grounds together.
4. Power the Mega and confirm the firmware starts; then connect the regulated servo supply.
5. Test one unloaded joint at a time with conservative limits before enabling normal movement.
6. Use the GUI's **Emergency Stop** to hold current servo outputs. It does not remove servo power; use the physical power switch to remove power in a dangerous condition. **Reset servos** returns joints to configured defaults and clears the firmware stop latch.

See [PROTOCOL.md](PROTOCOL.md) for command semantics and [PROJECT_GUIDE.md](PROJECT_GUIDE.md) for build, code structure, and runtime behavior.
