# Power Budget and Supply Selection

Guide for calculating servo power requirements and selecting an adequate power supply.

## Overview

This document explains:
- How to estimate servo current consumption
- How to calculate total power requirements
- How to select an appropriate power supply
- Safety margins and considerations

## Servo Current Consumption

Servo current varies based on:
- **Static current**: Current when servo is holding position (no load)
- **Running current**: Current when servo is moving
- **Stall current**: Current when servo is blocked from moving (maximum)
- **Load**: Mechanical load on the servo
- **Voltage**: Operating voltage (typically 4.8V-6V)

### Typical Servo Current Ranges

These are typical ranges for standard hobby servos. **Your servos may differ** - always check your servo's datasheet.

| Servo Type          | Static Current | Running Current | Stall Current |
|---------------------|----------------|-----------------|---------------|
| Micro servo (9g)    | 5-10 mA        | 100-200 mA      | 300-500 mA    |
| Standard servo      | 10-20 mA       | 200-400 mA      | 500-800 mA    |
| High-torque servo   | 20-50 mA       | 400-800 mA      | 1-2 A         |
| Metal gear servo    | 30-60 mA       | 500-1000 mA     | 1.5-3 A       |

**Important**: These are approximate values. You must measure or consult your specific servo's datasheet for accurate values.

## Calculating Your Power Requirements

### Step 1: Identify Your Servo Type

Determine what type of servos you're using:
- What is the servo weight/size?
- What is the rated torque?
- What is the rated voltage?
- Is the datasheet available?

### Step 2: Find Current Specifications

Check your servo's datasheet for:
- **No-load current** (or static current)
- **Running current** under typical load
- **Stall current** (maximum)

If no datasheet is available:
- Measure with a multimeter if possible
- Use conservative estimates from the table above
- Assume worst-case scenario

### Step 3: Calculate Total Current

For 14 servos, calculate:

**Scenario A: All servos holding position (static)**
```
Total Static Current = (Servo Static Current) × 14
```

**Scenario B: Multiple servos moving simultaneously**
```
Total Running Current = (Servo Running Current) × (Number of Moving Servos)
```

Typical humanoid robot might have 4-6 servos moving at once during walking or arm movements.

**Scenario C: Worst-case (all servos under load)**
```
Total Stall Current = (Servo Stall Current) × 14
```

This is unlikely but important for safety margin.

### Step 4: Add Safety Margin

Always add a safety margin of 50-100%:

```
Required Power Supply Current = (Total Current) × 1.5 to 2.0
```

This accounts for:
- Current spikes during movement
- Unexpected loads
- Power supply efficiency
- Future expansion

## Example Calculations

### Example 1: Standard Hobby Servos

Assume 14 standard servos with these specs:
- Static current: 15 mA
- Running current: 300 mA
- Stall current: 600 mA

**Static (all holding)**:
```
15 mA × 14 = 210 mA
```

**Running (6 servos moving)**:
```
300 mA × 6 = 1.8 A
```

**Worst-case (all stalled)**:
```
600 mA × 14 = 8.4 A
```

**Recommended power supply**:
```
1.8 A × 2.0 (safety margin) = 3.6 A minimum
5 A recommended for safety
```

### Example 2: High-Torque Servos

Assume 14 high-torque servos:
- Static current: 40 mA
- Running current: 600 mA
- Stall current: 1.5 A

**Static (all holding)**:
```
40 mA × 14 = 560 mA
```

**Running (6 servos moving)**:
```
600 mA × 6 = 3.6 A
```

**Worst-case (all stalled)**:
```
1.5 A × 14 = 21 A
```

**Recommended power supply**:
```
3.6 A × 2.0 (safety margin) = 7.2 A minimum
10 A recommended for safety
```

### Example 3: Mixed Servo Types

If using different servo types:

- 6 leg servos (high-torque): 600 mA running each
- 4 arm servos (standard): 300 mA running each
- 2 claw servos (micro): 150 mA running each
- 2 head servos (micro): 150 mA running each

**Running (typical movement)**:
```
(6 × 600 mA) + (4 × 300 mA) + (2 × 150 mA) + (2 × 150 mA)
= 3.6 A + 1.2 A + 0.3 A + 0.3 A
= 5.4 A
```

**Recommended power supply**:
```
5.4 A × 2.0 = 10.8 A minimum
12-15 A recommended
```

## Power Supply Selection

### Voltage Requirements

Most hobby servos operate at:
- 4.8V (4 AA NiMH batteries)
- 5V (regulated)
- 6V (5 AA NiMH batteries or regulated)

**Check your servo's rated voltage** before selecting a power supply.

Using higher voltage than rated can damage servos.
Using lower voltage will reduce torque and speed.

### Current Rating

Based on your calculations above, select a power supply with:
- Current rating ≥ calculated requirement with safety margin
- Continuous current rating (not peak)
- Over-current protection (recommended)

### Power Supply Types

#### 1. DC Bench Power Supply (Recommended)
- Adjustable voltage (set to 5V or 6V)
- High current (5A-20A available)
- Current limiting (protects servos)
- Over-current protection
- Voltage display
- Relatively expensive

#### 2. Fixed 5V Power Supply
- 5V fixed output
- High current models available (5A-10A)
- Simpler, less expensive
- No current limiting (add fuse)
- Ensure voltage is regulated

#### 3. Battery Pack
- NiMH AA batteries (4-5 cells for 4.8V-6V)
- LiPo battery with BEC (Battery Eliminator Circuit)
- Portable
- Voltage drops as battery discharges
- Need charging system
- Add low-voltage cutoff

#### 4. Computer Power Supply (ATX)
- Can provide 5V at high current (20A+)
- Inexpensive if repurposed
- Requires modification for use
- Need to wire properly
- Can be dangerous if mishandled

### Recommendations

**For beginners**:
- 5V 10A regulated DC power supply
- With current limiting if possible
- Fuse in series for protection

**For advanced users**:
- Adjustable DC bench power supply
- Set to servo rated voltage
- Use current limiting for safety

**For portable use**:
- High-capacity NiMH battery pack
- With low-voltage cutoff
- Monitor voltage during use

## Measuring Actual Current

To determine your actual power requirements:

### Method 1: Multimeter (Static)

1. Set multimeter to DC current mode (200 mA or 10A range)
2. Connect in series with power supply positive lead
3. Power on servos (disabled, holding position)
4. Measure current with all servos at home position
5. This is your static current

### Method 2: Multimeter (Dynamic)

1. Same setup as above
2. Enable servos and move them one at a time
3. Measure current during movement
4. Try moving multiple servos simultaneously
5. Record maximum current observed

### Method 3: Current Clamp (Best)

If you have a current clamp meter:
1. Clamp around power supply positive wire
2. Measure current during various operations
3. Easy to measure dynamic current
4. No need to break circuit

## Power Distribution

### Distribution Board

A servo distribution board is recommended:
- Provides common power rail for all servos
- Makes wiring easier
- Often includes capacitors for filtering
- May include LED indicators

### Wiring Considerations

- Use adequate wire gauge for current
- 18-22 AWG for 5-10A
- Keep wires short to reduce voltage drop
- Use multiple connections if current is very high
- Ensure good connections (crimp or solder)

### Capacitors

Add capacitors to smooth current spikes:
- 1000µF electrolytic capacitor across power rails
- 0.1µF ceramic capacitor for high-frequency noise
- Place near servo connectors

## Safety Considerations

### Over-Current Protection

Add protection to prevent damage:
- Fuse in series with power supply (rated below max current)
- Circuit breaker for easy reset
- Power supply with built-in current limiting

### Thermal Considerations

- Power supplies can get hot at high current
- Ensure adequate ventilation
- Don't enclose power supply without cooling
- Monitor temperature during use

### Voltage Drop

- Long wires cause voltage drop
- Voltage drop reduces servo torque
- Use thicker wires for long runs
- Measure voltage at servos under load

### Emergency Shutdown

- Install easily accessible power switch
- Quick disconnect in case of problems
- Keep switch reachable while testing

## Calculating Power (Watts)

Power (Watts) = Voltage (Volts) × Current (Amps)

For 5V at 5A:
```
5V × 5A = 25 Watts
```

For 6V at 10A:
```
6V × 10A = 60 Watts
```

Use this to:
- Select power supply wattage rating
- Estimate heat generation
- Size power wires

## Budget Recommendations

### Minimum Viable Setup

- 14 standard hobby servos
- 5V 5A regulated power supply
- Basic distribution board
- Total: ~$50-100

### Recommended Setup

- 14 standard to high-torque servos
- 5V 10A regulated power supply with current limiting
- Quality distribution board with capacitors
- Power switch and fuse
- Total: ~$100-200

### High-Performance Setup

- 14 high-torque metal gear servos
- Adjustable bench power supply (0-15V, 10-20A)
- Professional distribution board
- Current clamp meter for monitoring
- Total: ~$200-500

## Summary

1. **Check your servo datasheet** for current specifications
2. **Calculate total current** for your typical use case
3. **Add 50-100% safety margin**
4. **Select power supply** with adequate current and correct voltage
5. **Add protection** (fuse, current limiting)
6. **Measure actual current** when possible
7. **Monitor during use** for overheating or voltage drop

**Never guess on power requirements** - inadequate power supply can cause:
- Servo behavior issues (jitter, weak movement)
- Brownouts (Arduino resets)
- Power supply overheating
- Damage to components

When in doubt, choose a higher current rating power supply. It's better to have extra capacity than not enough.
