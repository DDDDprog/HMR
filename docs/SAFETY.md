# Safety Procedures

Critical safety information for the humanoid robot project.

## Overview

This document covers:
- Startup safety procedures
- Servo calibration
- Mechanical support requirements
- Emergency procedures
- Common hazards and how to avoid them

## Critical Safety Rules

1. **NEVER power servos from Arduino 5V pin** - This will damage the Arduino
2. **ALWAYS use external regulated power supply for servos**
3. **Keep power switch accessible for emergency shutdown**
4. **Support the robot mechanically during initial tests**
5. **Calibrate servo limits before full movement**
6. **Disconnect power before making wiring changes**
7. **Never leave powered robot unattended**
8. **Keep hands and loose objects away from moving parts**

## Startup Safety

### Initial Power-Up Sequence

1. **Verify All Connections**
   - Check all wiring matches [docs/PIN_MAP.md](PIN_MAP.md)
   - Verify common ground is connected
   - Ensure external power is rated for total servo current
   - Check no wires are loose or shorted

2. **Upload Firmware Without External Power**
   - Connect Arduino to computer via USB only
   - Do NOT connect external servo power yet
   - Upload firmware via Arduino IDE
   - Verify upload succeeds

3. **Test Serial Communication**
   - Run: `cargo run -- /dev/ttyACM0 ping`
   - Should receive "PONG" response
   - This confirms Arduino is working correctly

4. **Apply External Power**
   - Turn on external power supply
   - Verify voltage is correct (5V or 6V per servo specs)
   - Servos should NOT move (they start disabled)
   - Listen for any unusual sounds (hissing, crackling)

5. **Test MPU-6050**
   - Run: `cargo run -- /dev/ttyACM0 mpu`
   - Should receive non-zero sensor data
   - If all zeros, check I2C wiring

6. **Enable Servos**
   - Ensure robot is mechanically supported
   - Run: `cargo run -- /dev/ttyACM0 enable`
   - Servos attach but remain at home position
   - Listen for servo hum (normal)

7. **Test One Servo at a Time**
   - Start with a small movement
   - Run: `cargo run -- /dev/ttyACM0 set 0 10` (small angle)
   - Observe servo movement
   - Check for smooth motion, no binding
   - Return to home: `cargo run -- /dev/ttyACM0 set 0 0`
   - Repeat for each servo

### What NOT to Do on Startup

- ❌ Do NOT enable all servos and send large movements immediately
- ❌ Do NOT assume default angle limits are correct for your robot
- ❌ Do NOT test without mechanical support
- ❌ Do NOT leave robot unattended while powered
- ❌ Do NOT touch moving servos
- ❌ Do NOT bypass safety features

## Servo Calibration

### Why Calibration is Necessary

The firmware uses conservative default angle limits:
- Hip, knee, shoulder, elbow, claw: -90 to 90 degrees
- Ankle, head tilt: -45 to 45 degrees
- Head pan: -90 to 90 degrees

These may NOT match your actual mechanical limits. Moving beyond mechanical limits can:
- Strip servo gears
- Damage linkages
- Cause robot to fall
- Break servos

### Calibration Procedure

**Perform this BEFORE any full movement:**

1. **Prepare Robot**
   - Place robot on a stable surface
   - Secure robot so it cannot fall
   - Ensure all linkages are loose (not binding)
   - Have emergency power switch ready

2. **Calibrate One Servo at a Time**

   For each servo (0-13):

   a. **Enable servo**: 
      ```
      cargo run -- /dev/ttyACM0 enable
      ```

   b. **Test current limit**:
      - Try positive limit: `cargo run -- /dev/ttyACM0 set <index> 90`
      - Observe if servo hits mechanical stop
      - If it hits stop before 90°, note the actual maximum
      - Return to home: `cargo run -- /dev/ttyACM0 set <index> 0`

   c. **Test negative limit**:
      - Try negative limit: `cargo run -- /dev/ttyACM0 set <index> -90`
      - Observe if servo hits mechanical stop
      - If it hits stop before -90°, note the actual minimum
      - Return to home: `cargo run -- /dev/ttyACM0 set <index> 0`

   d. **Fine-tune limits**:
      - Adjust angle in 5-degree increments
      - Find exact mechanical limits
      - Subtract 5-10 degrees for safety margin
      - Record actual min/max for each servo

3. **Update Firmware Limits**

   Edit `arduino/humanoid_robot.ino`:
   ```cpp
   ServoLimits SERVO_LIMITS[NUM_SERVOS] = {
       { <actual_min>, <actual_max>, 0 },  // left hip
       { <actual_min>, <actual_max>, 0 },  // left knee
       // ... update all 14 servos
   };
   ```

4. **Update Rust Limits**

   Edit `src/limits.rs`:
   ```rust
   pub const SERVO_LIMITS: [ServoLimits; NUM_SERVOS] = [
       ServoLimits { min_angle: <actual_min>, max_angle: <actual_max>, home_angle: 0 },
       // ... update all 14 servos
   ];
   ```

5. **Re-upload and Re-test**
   - Re-upload Arduino firmware
   - Rebuild Rust application
   - Test limits again to verify

### Calibration Tips

- **Go slow**: Small movements, observe carefully
- **Listen for binding**: Grinding or straining sounds
- **Feel for resistance**: If servo struggles, stop immediately
- **Use safety margin**: Stay 5-10 degrees from mechanical limits
- **Document limits**: Write down actual limits for each servo
- **Re-calibrate if changes**: Any mechanical changes require recalibration

## Mechanical Support

### Why Support is Needed

During initial testing:
- Robot may not be balanced
- Servo movements can cause robot to fall
- Sudden movements can shift center of mass
- Calibration requires testing limits
- Robot may try impossible poses

### Support Methods

#### Method 1: Stand/Cradle

- Build a wooden or metal stand
- Cradle robot body securely
- Allow legs to hang freely
- Prevent tipping but allow movement

#### Method 2: Hanging

- Suspend robot from above
- Use strong cord or cable
- Allow freedom of movement
- Ensure secure attachment points

#### Method 3: Fixture

- Bolt robot to a test fixture
- Use rigid mounting points
- Prevent all movement except tested servo
- Best for single-servo testing

#### Method 4: Human Support

- Hold robot securely by body
- Keep hands away from moving parts
- Be prepared to catch if robot shifts
- Have helper control computer

### When Support is Required

- **Always during initial calibration**
- **During limit testing**
- **When testing new movements**
- **When adjusting firmware**
- **When robot is not on stable ground**

### When Support May Not Be Required

- After calibration is complete
- When robot is on stable, level surface
- When using tested, safe movements
- When robot is properly balanced
- **Still recommended to have support nearby**

## Emergency Procedures

### Servo Runaway

**Symptoms**:
- Servo moving uncontrollably
- Robot making unexpected movements
- Servos moving to extreme positions

**Actions**:
1. Immediately turn off external power supply
2. If unsafe, unplug power directly
3. Disconnect USB if needed
4. Do NOT try to stop moving servos with hands
5. Investigate cause (firmware bug, loose wire, etc.)

### Servo Binding/Straining

**Symptoms**:
- Servo making grinding noise
- Servo not reaching target position
- Servo vibrating or jittering
- Robot not moving as expected

**Actions**:
1. Immediately send: `cargo run -- /dev/ttyACM0 disable`
2. Turn off external power
3. Check for mechanical obstruction
4. Verify linkage not binding
5. Check servo angle limits
6. Re-calibrate if needed

### Electrical Issues

**Symptoms**:
- Burning smell
- Smoke
- Sparks
- Arduino or servos hot to touch

**Actions**:
1. Immediately disconnect all power
2. Unplug USB from computer
3. Do NOT touch hot components
4. Allow to cool before inspection
5. Check for short circuits
6. Inspect wiring for damage
7. Replace damaged components

### Robot Falling

**Symptoms**:
- Robot losing balance
- Robot tipping over
- Center of mass shifting dangerously

**Actions**:
1. Immediately turn off external power
2. Catch robot if safe to do so
3. Move robot away from edges
4. Inspect for damage
5. Check servo mounting
6. Adjust movements to be more stable

### Fire

**Symptoms**:
- Visible flames
- Burning smell
- Smoke

**Actions**:
1. Immediately disconnect all power
2. Use fire extinguisher (CO2 or dry chemical)
3. Do NOT use water on electrical fire
4. Evacuate if needed
5. Call emergency services if unsafe
6. Do not reuse damaged components

## Common Hazards

### Electrical Hazards

**Hazard**: Electric shock from power supply

**Prevention**:
- Use insulated power supply
- Keep power supply dry
- Don't touch live terminals
- Use proper connectors
- Disconnect power before working on wiring

**Hazard**: Short circuits causing fire

**Prevention**:
- Check wiring for shorts before power-up
- Use fuses or circuit breakers
- Don't exceed wire current rating
- Keep wires organized and secured
- Use strain relief on connections

### Mechanical Hazards

**Hazard**: Pinch points from moving servos

**Prevention**:
- Keep hands away from moving parts
- Use guards or covers if possible
- Be aware of servo horn location
- Don't wear loose clothing near robot
- Tie back long hair

**Hazard**: Robot falling and causing injury

**Prevention**:
- Always support robot during testing
- Keep robot away from edges
- Test on stable, level surface
- Have emergency power switch ready
- Don't place robot above valuable items

**Hazard**: Sharp edges or protrusions

**Prevention**:
- File or sand sharp edges
- Cover protruding screws
- Use protective caps
- Wear gloves when handling

### Thermal Hazards

**Hazard**: Servos overheating

**Prevention**:
- Monitor servo temperature
- Don't stall servos (prevent binding)
- Use adequate power supply
- Allow cooling time between tests
- Don't exceed servo duty cycle

**Hazard**: Power supply overheating

**Prevention**:
- Ensure adequate ventilation
- Don't enclose power supply
- Monitor temperature
- Don't exceed current rating
- Use thermal protection if available

## Safety Equipment

### Recommended Equipment

- **Fire extinguisher**: CO2 or dry chemical type
- **Safety glasses**: Protect eyes from debris
- **Gloves**: Insulated for electrical work
- **Multimeter**: For checking voltages and currents
- **Emergency power switch**: Easily accessible
- **First aid kit**: For minor injuries
- **Thermal camera (optional)**: For monitoring temperatures

### Optional Equipment

- **Current clamp meter**: For measuring servo current
- **Oscilloscope**: For debugging electrical issues
- **Thermal cutoff**: Automatic power-off at high temperature
- **Smoke detector**: Early warning of electrical issues

## Safe Working Practices

### Before Power-On

- [ ] Verify all wiring matches documentation
- [ ] Check for loose connections
- [ ] Test for short circuits with multimeter
- [ ] Ensure power supply is rated correctly
- [ ] Verify emergency switch is accessible
- [ ] Secure robot mechanically
- [ ] Clear work area of hazards
- [ ] Have safety equipment nearby

### During Operation

- [ ] Never leave robot unattended while powered
- [ ] Keep hands away from moving parts
- [ ] Monitor for unusual sounds or smells
- [ ] Watch for servo overheating
- [ ] Keep emergency switch reachable
- [ ] Test movements gradually
- [ ] Stop immediately if something seems wrong

### After Power-Off

- [ ] Disable servos before disconnecting power
- [ ] Move servos to safe position
- [ ] Turn off external power
- [ ] Wait for components to cool
- [ ] Inspect for any damage
- [ ] Document any issues
- [ ] Store robot safely

## Specific Risks for This Project

### Power Supply Risks

- Using inadequate current rating can cause brownouts
- Connecting servos to Arduino 5V will damage Arduino
- Poor ground connection causes erratic behavior
- Voltage drop causes weak servo movement

### Servo Risks

- Exceeding angle limits strips gears
- Stalling servos causes overheating
- Incorrect mounting causes binding
- Loose servo horns cause detachment

### MPU-6050 Risks

- Incorrect voltage damages sensor
- Poor I2C connection causes no data
- Sensor failure not detected by firmware
- Data may be noisy or inaccurate

### Software Risks

- Bugs in firmware can cause runaway
- Incorrect limits in code can damage servos
- Serial communication errors cause wrong commands
- Race conditions in software

## Training and Supervision

### For Beginners

- Read all documentation before starting
- Start with one servo at a time
- Have experienced supervision if possible
- Practice with disabled servos first
- Understand each command before using

### For Advanced Users

- Still follow safety procedures
- Don't skip calibration
- Test new code thoroughly
- Document modifications
- Keep safety in mind when optimizing

## Children and Pets

**Keep robot away from children and pets:**
- Moving parts can cause injury
- Electrical components are hazardous
- Robot can fall and cause damage
- Small parts are choking hazards

## Liability

This project is provided for educational purposes. The authors are not responsible for:
- Injury or damage caused by improper use
- Damage to components from incorrect wiring
- Injury from moving parts
- Fire or electrical incidents
- Any consequences of using this project

Use at your own risk. Follow all safety procedures. Seek professional help if unsure.

## Safety Checklist

Print this checklist and use it before each session:

### Pre-Session Checklist

- [ ] Wiring verified against documentation
- [ ] Power supply rated correctly
- [ ] Emergency switch accessible
- [ ] Robot mechanically supported
- [ ] Work area clear of hazards
- [ ] Safety equipment nearby
- [ ] Servo limits calibrated
- [ ] Firmware updated
- [ ] No loose clothing or jewelry
- [ ] Hair tied back if long

### During Session Checklist

- [ ] Never leave unattended
- [ ] Monitor for unusual sounds/smells
- [ ] Keep hands away from moving parts
- [ ] Test movements gradually
- [ ] Stop if something seems wrong
- [ ] Keep emergency switch reachable

### Post-Session Checklist

- [ ] Disable servos
- [ ] Move to safe position
- [ ] Turn off external power
- [ ] Disconnect USB
- [ ] Wait for cooling
- [ ] Inspect for damage
- [ ] Document issues
- [ ] Store safely

## Emergency Contacts

Keep these numbers handy:

- Local emergency services: 911 (or your local number)
- Poison control: (if using chemicals)
- Fire department: (if fire risk)
- Electrician: (for electrical issues)

## Reporting Safety Issues

If you discover a safety issue with this project:
1. Stop using the robot immediately
2. Document the issue
3. Report it to the project maintainers
4. Warn other users if appropriate
5. Do not use until issue is resolved

## Remember

**Safety is more important than functionality.**

It's better to:
- Move slower
- Test more thoroughly
- Ask for help
- Abort a test
- Take extra time

Than to:
- Damage components
- Cause injury
- Start a fire
- Destroy the robot

When in doubt, stop and ask for help.
