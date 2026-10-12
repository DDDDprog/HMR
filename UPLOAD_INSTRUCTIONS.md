# Arduino Firmware Upload Instructions

## Method 1: Using Arduino IDE (Recommended)

### Step 1: Open Arduino IDE
```bash
arduino &
```

### Step 2: Open the Firmware
1. File → Open
2. Navigate to: `/home/kali/index/ROBOT/arduino/humanoid_robot.ino`
3. Click Open

### Step 3: Select Board
1. Tools → Board → Arduino Mega 2560

### Step 4: Select Port
1. Tools → Port → /dev/ttyACM0

### Step 5: Upload
1. Click the Upload button (right arrow icon) or press Ctrl+U
2. Wait for "Done uploading" message

### Step 6: Test
```bash
cargo run --release -- /dev/ttyACM0 ping
```

Should return: `Pong`

## Method 2: Using Arduino CLI (if installed)

If you have arduino-cli installed:

```bash
# Install arduino-cli if not present
# Then:
arduino-cli compile --fqbn arduino:avr:mega arduino/humanoid_robot.ino
arduino-cli upload -p /dev/ttyACM0 --fqbn arduino:avr:mega arduino/humanoid_robot.ino
```

## Troubleshooting

### Port not found
```bash
# Check available ports
ls /dev/ttyACM* /dev/ttyUSB*

# If empty, check USB connection
dmesg | tail -20
```

### Permission denied
```bash
# Add user to dialout group
sudo usermod -a -G dialout $USER
# Log out and back in
```

### Upload fails
- Check USB cable (must be data cable, not charge-only)
- Try different USB port
- Ensure Arduino is powered (LED should be on)
- Check board selection is correct (Mega 2560)

## After Upload

Test the connection:
```bash
# Test ping
cargo run --release -- /dev/ttyACM0 ping

# Test status
cargo run --release -- /dev/ttyACM0 status

# Interactive mode
cargo run --release -- /dev/ttyACM0 interactive
```

## Next Steps

1. Upload firmware successfully
2. Verify with ping command
3. Wire hardware (without external power first)
4. Test with external power
5. Calibrate servo limits
