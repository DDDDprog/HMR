// Servo angle limits validation
// These must match the Arduino firmware limits

const NUM_SERVOS: usize = 14;

#[derive(Debug, Clone, Copy)]
pub struct ServoLimits {
    pub min_angle: i32,
    pub max_angle: i32,
    pub home_angle: i32,
}

// Conservative initial angle limits (degrees)
// These must be calibrated to actual mechanical limits
pub const SERVO_LIMITS: [ServoLimits; NUM_SERVOS] = [
    ServoLimits {
        min_angle: -90,
        max_angle: 90,
        home_angle: 0,
    }, // 0: left hip
    ServoLimits {
        min_angle: -90,
        max_angle: 90,
        home_angle: 0,
    }, // 1: left knee
    ServoLimits {
        min_angle: -45,
        max_angle: 45,
        home_angle: 0,
    }, // 2: left ankle
    ServoLimits {
        min_angle: -90,
        max_angle: 90,
        home_angle: 0,
    }, // 3: right hip
    ServoLimits {
        min_angle: -90,
        max_angle: 90,
        home_angle: 0,
    }, // 4: right knee
    ServoLimits {
        min_angle: -45,
        max_angle: 45,
        home_angle: 0,
    }, // 5: right ankle
    ServoLimits {
        min_angle: -90,
        max_angle: 90,
        home_angle: 0,
    }, // 6: left shoulder
    ServoLimits {
        min_angle: -90,
        max_angle: 90,
        home_angle: 0,
    }, // 7: left elbow
    ServoLimits {
        min_angle: -90,
        max_angle: 90,
        home_angle: 0,
    }, // 8: left claw/wrist
    ServoLimits {
        min_angle: -90,
        max_angle: 90,
        home_angle: 0,
    }, // 9: right shoulder
    ServoLimits {
        min_angle: -90,
        max_angle: 90,
        home_angle: 0,
    }, // 10: right elbow
    ServoLimits {
        min_angle: -90,
        max_angle: 90,
        home_angle: 0,
    }, // 11: right claw/wrist
    ServoLimits {
        min_angle: -90,
        max_angle: 90,
        home_angle: 0,
    }, // 12: head pan
    ServoLimits {
        min_angle: -45,
        max_angle: 45,
        home_angle: 0,
    }, // 13: head tilt
];

pub const SERVO_NAMES: [&str; NUM_SERVOS] = [
    "left hip",
    "left knee",
    "left ankle",
    "right hip",
    "right knee",
    "right ankle",
    "left shoulder",
    "left elbow",
    "left claw/wrist",
    "right shoulder",
    "right elbow",
    "right claw/wrist",
    "head pan",
    "head tilt",
];

/// Validate an angle against servo limits
pub fn validate_angle(index: usize, angle: i32) -> Result<(), String> {
    if index >= NUM_SERVOS {
        return Err(format!("Invalid servo index: {}", index));
    }

    let limits = SERVO_LIMITS[index];
    if angle < limits.min_angle || angle > limits.max_angle {
        return Err(format!(
            "Angle {} out of bounds for {}: min={}, max={}",
            angle, SERVO_NAMES[index], limits.min_angle, limits.max_angle
        ));
    }

    Ok(())
}

/// Get the home angle for a servo
#[allow(dead_code)]
pub fn get_home_angle(index: usize) -> Result<i32, String> {
    if index >= NUM_SERVOS {
        return Err(format!("Invalid servo index: {}", index));
    }
    Ok(SERVO_LIMITS[index].home_angle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_angle_valid() {
        assert!(validate_angle(0, 0).is_ok());
        assert!(validate_angle(0, -90).is_ok());
        assert!(validate_angle(0, 90).is_ok());
    }

    #[test]
    fn test_validate_angle_invalid() {
        assert!(validate_angle(0, -91).is_err());
        assert!(validate_angle(0, 91).is_err());
    }

    #[test]
    fn test_validate_angle_invalid_index() {
        assert!(validate_angle(14, 0).is_err());
        assert!(validate_angle(100, 0).is_err());
    }

    #[test]
    fn test_get_home_angle() {
        assert_eq!(get_home_angle(0).unwrap(), 0);
        assert_eq!(get_home_angle(5).unwrap(), 0);
    }

    #[test]
    fn test_get_home_angle_invalid() {
        assert!(get_home_angle(14).is_err());
    }

    #[test]
    fn test_ankle_limits_stricter() {
        // Ankle servos have stricter limits (-45 to 45)
        assert!(validate_angle(2, 45).is_ok());
        assert!(validate_angle(2, -45).is_ok());
        assert!(validate_angle(2, 46).is_err());
        assert!(validate_angle(2, -46).is_err());
    }
}
