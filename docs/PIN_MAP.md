# Pin Map - Arduino Mega 2560

Complete pin assignments for the humanoid robot project.

## Servo Signal Pins

| Arduino Pin | Servo Name     | Wire Color (Suggested) | Function          |
|-------------|----------------|------------------------|-------------------|
| D2          | Left Hip       | Yellow/Orange         | PWM Signal        |
| D3          | Left Knee      | Yellow/Orange         | PWM Signal        |
| D4          | Left Ankle     | Yellow/Orange         | PWM Signal        |
| D5          | Right Hip      | Yellow/Orange         | PWM Signal        |
| D6          | Right Knee     | Yellow/Orange         | PWM Signal        |
| D7          | Right Ankle    | Yellow/Orange         | PWM Signal        |
| D8          | Left Shoulder  | Yellow/Orange         | PWM Signal        |
| D9          | Left Elbow     | Yellow/Orange         | PWM Signal        |
| D10         | Left Claw      | Yellow/Orange         | PWM Signal        |
| D11         | Right Shoulder | Yellow/Orange         | PWM Signal        |
| D12         | Right Elbow    | Yellow/Orange         | PWM Signal        |
| D13         | Right Claw     | Yellow/Orange         | PWM Signal        |
| D22         | Head Pan       | Yellow/Orange         | PWM Signal        |
| D23         | Head Tilt      | Yellow/Orange         | PWM Signal        |

**Servo Wire Standard** (most hobby servos):
- **Signal (S)**: Yellow, Orange, or White - Connect to Arduino pins above
- **Power (V+)**: Red - Connect to external servo power supply positive
- **Ground (GND)**: Brown or Black - Connect to external power ground AND Arduino ground

## MPU-6050 IMU Pins

| MPU-6050 Pin | Arduino Pin | Wire Color (Suggested) | Function          |
|--------------|-------------|------------------------|-------------------|
| VCC          | 5V          | Red                    | Power (5V only if board supports it) |
| GND          | GND         | Black                  | Ground            |
| SDA          | D20 (SDA)   | Green                  | I2C Data          |
| SCL          | D21 (SCL)   | Blue                   | I2C Clock         |
| INT          | (unused)    | -                      | Interrupt (optional) |

**Important MPU-6050 Notes**:
- Some GY-521 boards require 3.3V instead of 5V - check your board
- If your board requires 3.3V, connect VCC to Arduino 3.3V pin
- I2C pins are fixed on Arduino Mega: SDA=D20, SCL=D21
- INT pin is optional and not currently used in firmware

## USB Serial Connection

| Arduino Pin | Function       | Connection           |
|-------------|----------------|----------------------|
| D0 (RX)     | Serial Receive | USB (do not connect) |
| D1 (TX)     | Serial Transmit| USB (do not connect) |

**Important**: D0 and D1 are used for USB serial communication with the computer. Do not connect anything else to these pins.

## Power Pins

| Arduino Pin | Function       | Connection                      |
|-------------|----------------|---------------------------------|
| 5V          | 5V Output      | MPU-6050 VCC (if 5V compatible) |
| 3.3V        | 3.3V Output    | MPU-6050 VCC (if 3.3V required) |
| GND         | Ground         | Common ground with external power |
| VIN         | Input Voltage  | External power supply (7-12V DC) |

**Critical**: Do NOT power servos from Arduino 5V pin. Use external regulated power supply.

## Reserved Pins (Do Not Use)

| Pin | Reason                           |
|-----|----------------------------------|
| D0  | USB Serial RX                    |
| D1  | USB Serial TX                    |
| D20 | I2C SDA (MPU-6050)               |
| D21 | I2C SCL (MPU-6050)               |

## Available/Unused Pins

The following Mega 2560 pins are not used in this project and can be used for future expansion:

| Analog Pins | Digital Pins |
|-------------|--------------|
| A0-A15      | D14-D19, D24-D53 |

## Pin Summary Table

| Pin | Function           | Device     | Direction |
|-----|--------------------|------------|-----------|
| D2  | Servo Signal       | Left Hip   | Output    |
| D3  | Servo Signal       | Left Knee  | Output    |
| D4  | Servo Signal       | Left Ankle | Output    |
| D5  | Servo Signal       | Right Hip  | Output    |
| D6  | Servo Signal       | Right Knee | Output    |
| D7  | Servo Signal       | Right Ankle| Output    |
| D8  | Servo Signal       | Left Shoulder| Output   |
| D9  | Servo Signal       | Left Elbow | Output    |
| D10 | Servo Signal       | Left Claw  | Output    |
| D11 | Servo Signal       | Right Shoulder| Output  |
| D12 | Servo Signal       | Right Elbow| Output    |
| D13 | Servo Signal       | Right Claw | Output    |
| D22 | Servo Signal       | Head Pan   | Output    |
| D23 | Servo Signal       | Head Tilt  | Output    |
| D20 | I2C Data           | MPU-6050   | Bidirectional |
| D21 | I2C Clock          | MPU-6050   | Output    |
| D0  | USB Serial RX      | Computer   | Input     |
| D1  | USB Serial TX      | Computer   | Output    |
| 5V  | Power (MPU only)   | MPU-6050   | Output    |
| GND | Common Ground      | All        | Reference |

## ASCII Wiring Diagram

```
                          Arduino Mega 2560
                         +------------------+
                         |                  |
      Servo Power -----  | VIN        GND  |-----+-- Common Ground
      (7-12V DC)        |                  |     |
                         |         5V  -----+----- MPU-6050 VCC
                         |                  |     |
                         | D2  D3  D4  D5   |     |
   Left Hip/Knee/Ankle--|----|--|--|--      |     |
                         |    |  |  |  |     |     |
                         | D6  D7  D8  D9   |     |
  Right Hip/Knee/Ankle--|----|--|--|--      |     |
                         |    |  |  |  |     |     |
                         | D10 D11 D12 D13  |     |
 Left/Right Arms/Claws--|----|--|--|--      |     |
                         |    |  |  |  |     |     |
                         | D22 D23          |     |
          Head Pan/Tilt--|----|-------------|     |
                         |                  |     |
                         | D20 D21          |     |
              MPU-6050--|----|-------------|     |
               (SDA/SCL) |                  |     |
                         | D0  D1           |     |
              USB Serial |--|--             |     |
                         |                  |     |
                         +------------------+     |
                                                      |
                         External Servo Power         |
                         +----------------+           |
                         | 5V Regulated  |-----------+-- Servo V+ (Red)
                         | Power Supply   |           |
                         | (Adequate      |           |
                         |  current)      |           |
                         |                |           |
                         | GND            |-----------+-- Servo GND (Brown/Black)
                         +----------------+           |
                                                      |
                               To All Servos:         |
                               Signal: Arduino pins  |
                               Power: External 5V     |
                               Ground: Common GND     |
```

## Servo Connection Details

Each servo has 3 wires:

1. **Signal Wire** (Yellow/Orange/White)
   - Connect to corresponding Arduino pin (D2-D13, D22-D23)
   - Carries PWM control signal

2. **Power Wire** (Red)
   - Connect to EXTERNAL servo power supply positive terminal
   - DO NOT connect to Arduino 5V pin

3. **Ground Wire** (Brown/Black)
   - Connect to external power supply ground
   - ALSO connect to Arduino GND pin (common ground required)

## Wiring Checklist

Before powering on:

- [ ] All 14 servo signal wires connected to correct Arduino pins
- [ ] All servo power wires connected to external power supply positive
- [ ] All servo ground wires connected to external power supply ground
- [ ] External power supply ground connected to Arduino GND
- [ ] MPU-6050 VCC connected to 5V or 3.3V (check board requirement)
- [ ] MPU-6050 GND connected to Arduino GND
- [ ] MPU-6050 SDA connected to Arduino D20
- [ ] MPU-6050 SCL connected to Arduino D21
- [ ] Nothing connected to Arduino D0 or D1
- [ ] Nothing connected to Arduino 5V except MPU-6050
- [ ] External power supply rated for total servo current
- [ ] Power switch accessible for emergency shutdown

See [docs/WIRING.md](WIRING.md) for detailed wiring instructions and power-up sequence.
