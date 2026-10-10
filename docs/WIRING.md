# Wiring Guide

Detailed wiring instructions for the humanoid robot project.

## Overview

This guide explains how to connect:
- 14 hobby servos to Arduino Mega 2560
- MPU-6050 IMU sensor to Arduino
- External power supply for servos
- Common ground connections
- USB connection to computer

## Safety First

**CRITICAL WARNINGS**:
1. NEVER power servos from Arduino 5V pin - this will damage the Arduino
2. Always use external regulated power supply for servos
3. Common ground is required between Arduino and external power
4. Disconnect power before making any wiring changes
5. Double-check all connections before powering on
6. Keep an emergency power switch accessible

## Required Components

- Arduino Mega 2560
- 14 hobby servos (standard PWM servos with 3 wires)
- GY-521 MPU-6050 IMU module
- External regulated 5V power supply (adequate current for all servos)
- USB cable (data cable, not charge-only)
- Breadboard or servo distribution board (optional but recommended)
- Jumper wires (male-to-male, male-to-female as needed)
- Wire strippers/cutters
- Multimeter (for testing)

## Servo Wiring

### Servo Wire Colors

Most hobby servos use this standard:

| Wire Color | Function          | Connect To              |
|------------|-------------------|-------------------------|
| Yellow/Orange/White | Signal | Arduino digital pins (D2-D13, D22-D23) |
| Red        | Power (+)         | External power supply positive (+) |
| Brown/Black | Ground (-)        | External power ground AND Arduino GND |

**Note**: Some servos may use different colors. Always check your servo's datasheet.

### Servo Connection Steps

1. **Prepare External Power Distribution**
   - If using a breadboard or distribution board, connect external power supply positive to the positive rail
   - Connect external power supply ground to the ground rail
   - This makes it easy to connect all servos

2. **Connect Each Servo**
   For each of the 14 servos:
   
   a. **Signal Wire** (Yellow/Orange/White):
      - Connect to the corresponding Arduino pin:
      - Left Hip → D2
      - Left Knee → D3
      - Left Ankle → D4
      - Right Hip → D5
      - Right Knee → D6
      - Right Ankle → D7
      - Left Shoulder → D8
      - Left Elbow → D9
      - Left Claw → D10
      - Right Shoulder → D11
      - Right Elbow → D12
      - Right Claw → D13
      - Head Pan → D22
      - Head Tilt → D23
   
   b. **Power Wire** (Red):
      - Connect to external power supply positive rail
      - DO NOT connect to Arduino 5V pin
   
   c. **Ground Wire** (Brown/Black):
      - Connect to external power ground rail
      - Also connect this ground rail to Arduino GND pin

3. **Verify Connections**
   - Use a multimeter to check for shorts between signal and power/ground
   - Ensure no two servo signal wires are connected together
   - Verify all ground connections are common

## MPU-6050 Wiring

### MPU-6050 Pinout

The GY-521 module typically has these pins (may vary by manufacturer):

| Pin Label | Function          | Connect To              |
|-----------|-------------------|-------------------------|
| VCC       | Power             | Arduino 5V or 3.3V (check board) |
| GND       | Ground            | Arduino GND             |
| SDA       | I2C Data          | Arduino D20 (SDA)       |
| SCL       | I2C Clock         | Arduino D21 (SCL)       |
| INT       | Interrupt         | Not used (leave unconnected) |
| XDA       | Auxiliary I2C Data| Not used (leave unconnected) |
| XCL       | Auxiliary I2C Clock| Not used (leave unconnected) |
| AD0       | I2C Address Select| Not used (leave unconnected) |

### MPU-6050 Connection Steps

1. **Check Your Board's Voltage Requirement**
   - Some GY-521 boards accept 5V
   - Some require 3.3V only
   - Check your board's documentation or markings
   - If unsure, use 3.3V to be safe

2. **Connect Power**
   - If 5V compatible: Connect VCC to Arduino 5V pin
   - If 3.3V only: Connect VCC to Arduino 3.3V pin
   - Connect GND to Arduino GND

3. **Connect I2C**
   - Connect SDA to Arduino D20 (marked SDA on Mega)
   - Connect SCL to Arduino D21 (marked SCL on Mega)

4. **Leave Unused Pins Disconnected**
   - INT, XDA, XCL, AD0 can be left unconnected
   - Do not connect them to anything

## External Power Supply

### Power Supply Requirements

See [docs/POWER_BUDGET.md](POWER_BUDGET.md) for detailed power calculations.

Key requirements:
- Regulated 5V output (or 6V for high-torque servos - check servo specs)
- Adequate current for all servos (typically 2-5A minimum for 14 servos)
- Stable output with low ripple
- Over-current protection recommended

### Power Supply Connection

1. **Choose a Suitable Power Supply**
   - 5V regulated DC power supply
   - Current rating ≥ 2A (higher is better)
   - Center-positive barrel connector or terminal block

2. **Connect to Servo Power Distribution**
   - Connect power supply positive (+) to servo power distribution positive rail
   - Connect power supply negative (-) to servo power distribution ground rail

3. **Connect Common Ground**
   - Connect power supply ground to Arduino GND pin
   - This is CRITICAL - without common ground, signals won't work properly

4. **Add Power Switch (Recommended)**
   - Install a switch between power supply and servo distribution
   - Allows quick power-off in emergencies
   - Make switch easily accessible

## USB Connection

### Arduino to Computer

1. **Use a Data Cable**
   - Must be a USB data cable, not a charge-only cable
   - Data cables have 4 wires; charge-only have 2 wires
   - Test with another device if unsure

2. **Connect to Computer**
   - Plug USB cable into Arduino's USB port
   - Plug other end into computer's USB port
   - Arduino should power on (LED lights)

3. **Verify Connection**
   - On Linux: Check `ls /dev/ttyACM*` or `ls /dev/ttyUSB*`
   - Arduino should appear as `/dev/ttyACM0` or similar

## Common Ground Connection

**This is the most critical connection**:

- Arduino GND pin must be connected to:
  - External power supply ground
  - MPU-6050 GND
  - All servo ground wires (via distribution board)

Without common ground:
- Servo signals will be unreliable
- MPU-6050 may not communicate
- Random behavior and errors

## Wiring Diagram (ASCII)

```
                           +-------------------------+
                           |   Arduino Mega 2560     |
                           |                         |
           USB Cable ----- | USB                     |
                           |                         |
                           | VIN        GND  -------+-----------------+
                           |                  |     |                 |
                           |         5V  -----+-----+-----------------+
                           |                  |     |                 |
                           | D2-D13, D22-D23  |     |                 |
           Servo Signals --|------------------+     |                 |
                           |                  |     |                 |
                           | D20 (SDA)  ------+-----+-----------------+
                           | D21 (SCL)  ------+     |                 |
                           |                  |     |                 |
                           +------------------+     |                 |
                                                      |                 |
                  +-----------------------------------+                 |
                  |                                                     |
    +-------------+-----------+         +-------------------------------+
    |  External Power Supply   |         |     Servo Distribution        |
    |                         |         |                               |
    |  (+) -------------------+---------|----+ Servo Power Rails (+)     |
    |                         |         |    |                           |
    |  (-) -------------------+---------|----+ Servo Ground Rails (-)----|---+
    |                         |         |    |                           |   |
    +-------------------------+         +----+---------------------------+   |
                                                                        |   |
                   Common Ground (All GND connected together) -----------+   |
                                                                            |
                                                                        +---+---+
                                                                        |       |
                                                                    +---+---+   |
                                                                    | Servos|   |
                                                                    |       |   |
                                                                    +-------+   |
                                                                                |
                                                                    +-----------+
                                                                    | MPU-6050  |
                                                                    |           |
                                                                    | VCC ------+ (to 5V or 3.3V)
                                                                    | GND ------+ (to common GND)
                                                                    | SDA ------+ (to D20)
                                                                    | SCL ------+ (to D21)
                                                                    +-----------+
```

## Power-Up Sequence

### Safe Power-Up Procedure

1. **Disconnect All Power**
   - Turn off external power supply
   - Unplug Arduino USB
   - Verify no power is connected

2. **Verify All Connections**
   - Double-check all wiring
   - Use multimeter to check for shorts
   - Ensure common ground is connected
   - Verify no wires are loose

3. **Upload Firmware First**
   - Connect Arduino USB to computer (no external power yet)
   - Upload firmware using Arduino IDE
   - Verify upload succeeds
   - Keep Arduino connected via USB

4. **Test Serial Communication**
   - Run desktop application: `cargo run -- /dev/ttyACM0 ping`
   - Should receive "PONG" response
   - If no response, check USB connection and port

5. **Apply External Power**
   - Turn on external power supply
   - Servos should NOT move (they start disabled)
   - Check power supply voltage is correct (5V or 6V)

6. **Test MPU-6050**
   - Run: `cargo run -- /dev/ttyACM0 mpu`
   - Should receive accelerometer and gyroscope data
   - If all zeros, check I2C wiring

7. **Test Servo Control (Carefully)**
   - Support the robot mechanically (hold it in place)
   - Run: `cargo run -- /dev/ttyACM0 enable`
   - Servos are now enabled but at home position
   - Test one servo at a time with small movements
   - Run: `cargo run -- /dev/ttyACM0 set 0 10` (small angle)
   - Verify servo moves correctly
   - Run: `cargo run -- /dev/ttyACM0 set 0 -10`
   - Continue testing each servo

8. **Calibrate Limits**
   - For each servo, find actual mechanical limits
   - Update limits in Arduino firmware and Rust code
   - See [docs/SAFETY.md](SAFETY.md) for calibration procedure

### Power-Down Sequence

1. **Disable Servos**
   - Run: `cargo run -- /dev/ttyACM0 disable`
   - Servos detach and relax

2. **Move to Safe Position**
   - Before disabling, move servos to a safe home position
   - Run: `cargo run -- /dev/ttyACM0 home`

3. **Turn Off External Power**
   - Switch off external power supply
   - Servos will be unpowered

4. **Disconnect USB**
   - Close desktop application
   - Unplug Arduino USB from computer

## Troubleshooting Wiring Issues

### Servos Not Moving

**Check**:
- External power is connected and on
- Servo power wires connected to external power (not Arduino 5V)
- Common ground connected
- Servos are enabled (send ENABLE command)
- Servo signal wires connected to correct pins

### Servos Jittering or Erratic

**Check**:
- Common ground connection
- Power supply adequate current
- Power supply voltage stable
- Signal wires not picking up noise (keep short)
- External interference (motors, etc.)

### MPU-6050 Not Detected

**Check**:
- SDA connected to D20, SCL to D21
- VCC connected to correct voltage (5V or 3.3V)
- GND connected to common ground
- I2C pull-up resistors present on GY-521 board
- Board is not damaged

### Arduino Not Detected by Computer

**Check**:
- USB cable is a data cable (not charge-only)
- Try different USB port
- Try different USB cable
- Check Arduino LED lights up
- On Linux: check `dmesg` for USB errors
- May need to add user to dialout group

### Servos Powering from Arduino 5V

**DANGEROUS** - If you accidentally connected servos to Arduino 5V:
- Immediately disconnect power
- Arduino may be damaged
- Do not use that Arduino for this project
- Rewire servos to external power supply

## Wire Management Tips

1. **Keep Signal Wires Short**
   - Longer wires pick up more noise
   - Use shortest practical length

2. **Separate Power and Signal**
   - Don't bundle power and signal wires together
   - Reduces electrical noise

3. **Use Proper Connectors**
   - Servo connectors are usually standard
   - Ensure good connections
   - Consider heat-shrink or tape for strain relief

4. **Label Wires**
   - Label which servo each wire belongs to
   - Makes debugging easier

5. **Secure Wires**
   - Use zip ties or cable management
   - Prevent wires from getting caught
   - Keep away from moving parts

## Next Steps

After wiring is complete:

1. Read [docs/SAFETY.md](SAFETY.md) for safety procedures
2. Read [docs/POWER_BUDGET.md](POWER_BUDGET.md) for power requirements
3. Calibrate servo limits per [docs/SAFETY.md](SAFETY.md)
4. Test with [docs/PROTOCOL.md](PROTOCOL.md) commands
