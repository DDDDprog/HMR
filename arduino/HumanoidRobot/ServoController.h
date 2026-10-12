#ifndef SERVO_CONTROLLER_H
#define SERVO_CONTROLLER_H

#include "Config.h"
#include <Servo.h>
#include <EEPROM.h>

struct ServoConfig {
  uint16_t min_pulse;
  uint16_t max_pulse;
  uint16_t center_pulse;
  uint16_t min_angle;  // *10 (0.1 degree resolution)
  uint16_t max_angle;  // *10
  bool reversed;
  uint8_t pin;
  char name[16];
};

class ServoController {
public:
  ServoController();
  
  bool begin();
  void update();
  
  // Direct control
  bool setAngle(uint8_t index, uint16_t angle_x10);  // angle * 10
  bool setPulse(uint8_t index, uint16_t pulse_us);
  bool setAllAngles(const uint16_t* angles_x10);
  
  // Getters
  uint16_t getAngle(uint8_t index) const;  // Returns angle * 10
  uint16_t getPulse(uint8_t index) const;
  bool isAttached(uint8_t index) const;
  const ServoConfig* getConfig(uint8_t index) const;
  
  // Limits configuration
  bool setLimits(uint8_t index, uint16_t min_angle_x10, uint16_t max_angle_x10,
                 uint16_t min_pulse, uint16_t max_pulse, uint16_t center_pulse);
  bool setReversed(uint8_t index, bool reversed);
  
  // Persistence
  bool saveConfig();
  bool loadConfig();
  void resetToDefaults();
  
  // Safety
  void emergencyStop();
  bool isEmergencyStopped() const;
  void releaseAll();
  bool checkSafety(uint8_t index, uint16_t angle_x10, uint16_t pulse_us) const;
  
  // Calibration helpers
  void calibrateCenter(uint8_t index);
  void calibrateMin(uint8_t index);
  void calibrateMax(uint8_t index);
  
  // Status
  uint32_t getLastUpdateMs() const;
  bool isMoving(uint8_t index) const;
  
private:
  Servo servos_[NUM_SERVOS];
  ServoConfig configs_[NUM_SERVOS];
  uint16_t current_angles_[NUM_SERVOS];  // *10
  uint16_t target_angles_[NUM_SERVOS];   // *10
  uint16_t current_pulses_[NUM_SERVOS];
  bool attached_[NUM_SERVOS];
  bool moving_[NUM_SERVOS];
  uint32_t last_update_ms_;
  bool emergency_stop_;
  
  // Interpolation for smooth motion
  uint16_t step_size_[NUM_SERVOS];  // Degrees * 10 per update
  
  uint16_t angleToPulse(uint8_t index, uint16_t angle_x10) const;
  uint16_t pulseToAngle(uint8_t index, uint16_t pulse_us) const;
  bool validateIndex(uint8_t index) const;
  void writeEEPROMConfig();
  bool readEEPROMConfig();
  void applyDefaultConfig(uint8_t index);
};

#endif // SERVO_CONTROLLER_H