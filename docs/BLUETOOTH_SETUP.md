# Bluetooth Setup

This project supports runtime serial control through a Bluetooth UART module such as HC-05 or HC-06. The host connects to the module's Bluetooth Serial Port Profile (SPP); the module's UART connects to the Mega's `Serial1` pins. Bluetooth is not used to flash the Mega firmware.

For the full servo signal-pin map, servo power precautions, and MPU-6050 connections, see [HARDWARE_WIRING.md](HARDWARE_WIRING.md).

## Recommended hardware

- Arduino Mega 2560
- HC-05 or HC-06 Bluetooth module
- 5V power source
- common GND
- separate servo power supply sized for the installed servos

## Wiring

| Bluetooth module | Arduino Mega                                        |
| ---------------- | --------------------------------------------------- |
| TX               | 19 (RX1)                                            |
| RX               | 18 (TX1) through a 5V-to-3.3V level shifter/divider |
| VCC              | 5V                                                  |
| GND              | GND                                                 |

Do not use servo PWM pins for the Bluetooth serial connection. TX/RX are crossed: module TX goes to Mega RX1, and Mega TX1 goes to module RX. The Mega transmits 5V logic; use a level shifter or resistor divider on the module RX line. Check the module breakout's VCC specification: some carrier boards accept 5V, while a bare HC-05 radio module is 3.3V-only. Keep all grounds common. Do not power the servo bank from the Mega's 5V pin.

## Module configuration

The module's UART baud rate must match the firmware. The wireless firmware default is 9600 baud, defined in `arduino/HumanoidRobot/Config.h`; the GUI default is also 9600. Bluetooth pairing PIN and UART baud are separate settings. If changing the module baud with its AT commands, also change `ROBOT_BAUD_RATE` and the GUI baud rate.

If your module uses a different baud rate, update:

```cpp
#define ROBOT_BAUD_RATE 9600
```

## Pairing

### Linux

Power the robot and put the HC-05/HC-06 into its normal discoverable mode. Start an interactive BlueZ session and find the module address while it is visible:

```bash
bluetoothctl
agent on
default-agent
scan on
# Wait for the HC-05/HC-06 to appear, then use its address while scanning
pair <MAC>
trust <MAC>
scan off
exit
```

If pairing asks for a PIN, enter the value configured on the module; common factory values are `1234` or `0000`, but they are not universal. Keep the module near the computer during setup. If BlueZ reports `Device not available`, scan again and issue `pair <MAC>` while the module is currently listed. `AuthenticationFailed` usually means the PIN is wrong, the module is not in pairing mode, or another host has claimed it.

Pairing alone does not necessarily create a serial device. Determine the module's SPP channel (channel 1 is common), then create the RFCOMM binding:

```bash
sudo rfcomm bind 0 <MAC> 1
ls -l /dev/rfcomm0
```

`rfcomm bind` creates the device binding; RFCOMM may remain idle until an application opens `/dev/rfcomm0`. Therefore, `rfcomm -a` showing a bound channel as closed before connecting the GUI does not by itself prove pairing failed. Refresh ports in the GUI, select `/dev/rfcomm0`, and press **Connect**. The GUI reports serial-port state separately from robot ping response.

If the bind command reports a permission error, run it with administrator privileges. If the GUI cannot open the device, check its owner/group with `ls -l /dev/rfcomm0`; add your user to the owning serial group if needed, then log out and back in. Release the binding later with `sudo rfcomm release 0`.

### Windows

- Pair the Bluetooth device in Windows Bluetooth settings
- In Device Manager, identify the outgoing serial COM port created for the module's SPP service
- Use that COM port in the desktop GUI

## Firmware flag

Enable the wireless build in:

```cpp
#define USE_BLUETOOTH_SERIAL 1
```

This selects `Serial1` (Mega pins 18/TX1 and 19/RX1) for runtime control at 9600 baud. With `0`, the firmware uses USB `Serial` at 115200 baud instead.

Firmware uploads use the Mega's USB bootloader regardless of the selected runtime serial transport. Connect USB for compilation/upload, then disconnect its data cable after flashing if you want wireless-only operation. Keep the robot and Bluetooth module powered independently.

## GUI config

Choose **Wireless Bluetooth** in the GUI link-mode selector. The application defaults are:

- Linux: `/dev/rfcomm0`
- Windows: `COM3`
- Bluetooth baud rate: `9600`

The GUI supports refresh/select for detected ports, manual port entry, and baud-rate editing while disconnected. **Wired USB** uses 115200 baud; **Wireless Bluetooth** uses 9600 baud. The selector sets these defaults, but the selected firmware must also match `USE_BLUETOOTH_SERIAL`; disconnect before changing modes. The GUI pings once per second while responsive to service the firmware watchdog. Connection is manual by default; use **Save app settings** to persist the host configuration. The config file is stored under the platform config directory in `humanoid-robot-gui/config.ron`.

## Testing

After pairing/binding and flashing firmware built with `USE_BLUETOOTH_SERIAL 1`, power the Mega, Bluetooth module, and servos appropriately. USB data may then be disconnected. In the GUI select **Wireless Bluetooth**, refresh ports, choose `/dev/rfcomm0` (or the Windows COM port), and connect. A successful **Robot: RESPONDING** state means the Mega returned a valid ping ACK; **Serial port: OPEN** alone only means the host opened a device.

For a wired session, flash firmware with `USE_BLUETOOTH_SERIAL 0`, connect the Mega USB port, then choose **Wired USB** and select the detected USB serial port. Changing the GUI mode does not change the firmware build; recompile and upload over USB when switching firmware transport modes.

If it does not respond:

- confirm the module is powered, discoverable/paired, and not connected to another host
- confirm the OS serial endpoint exists and the GUI selected that endpoint
- confirm the baud rates match on firmware, module UART, and GUI
- verify crossed TX/RX wiring, RX level shifting, and shared ground
- verify the Mega is powered and the latest firmware was flashed
- use the GUI's **Check robot** button after correcting the issue

## Notes

- USB is still supported by setting `USE_BLUETOOTH_SERIAL` to `0`.
- Bluetooth is intended for command and telemetry traffic, not for high-bandwidth video streaming.
