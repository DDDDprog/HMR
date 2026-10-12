#include "ServoController.h"
#include <Arduino.h>
#include <EEPROM.h>

const uint8_t servo_pins[NUM_SERVOS] = {
  PIN_LEFT_HIP, PIN_LEFT_KNEE, PIN_LEFT_ANKLE,
  PIN_RIGHT_HIP, PIN_RIGHT_KNEE, PIN_RIGHT_ANKLE,
  PIN_LEFT_SHOULDER, PIN_LEFT_ELBOW, PIN_LEFT_CLAW,
  PIN_RIGHT_SHOULDER, PIN_RIGHT_ELBOW, PIN_RIGHT_CLAW,
  PIN_HEAD_PAN, PIN_HEAD_TILT
};

const char* servo_names[NUM_SERVOS] = {
  "Left Hip", "Left Knee", "Left Ankle",
  "Right Hip", "Right Knee", "Right Ankle",
  "Left Shoulder", "Left Elbow", "Left Claw",
  "Right Shoulder", "Right Elbow", "Right Claw",
  "Head Pan", "Head Tilt"
};

ServoController::ServoController() 
  : last_update_ms_(0)
  , emergency_stop_(false) {
  for (int i = 0; i < NUM_SERVOS; i++) {
    current_angles_[i] = DEFAULT_CENTER * 10 / 10;  // Will be set properly in begin()
    target_angles_[i] = DEFAULT_CENTER * 10 / 10;
    current_pulses_[i] = DEFAULT_CENTER;
    attached_[i] = false;
    moving_[i] = false;
    step_size_[i] = 50;  // 5 degrees per update default
    memset(configs_[i].name, 0, sizeof(configs_[i].name));
  }
}

bool ServoController::begin() {
  for (int i = 0; i < NUM_SERVOS; i++) {
    applyDefaultConfig(i);
    configs_[i].pin = servo_pins[i];
    strncpy(configs_[i].name, servo_names[i], sizeof(configs_[i].name) - 1);
    
    servos_[i].attach(configs_[i].pin, configs_[i].min_pulse, configs_[i].max_pulse);
    attached_[i] = true;
    
    // Start at center
    current_pulses_[i] = configs_[i].center_pulse;
    current_angles_[i] = pulseToAngle(i, current_pulses_[i]);
    target_angles_[i] = current_angles_[i];
    servos_[i].writeMicroseconds(current_pulses_[i]);
  }
  
  loadConfig();
  last_update_ms_ = millis();
  emergency_stop_ = false;
  return true;
}

void ServoController::update() {
  if (emergency_stop_) return;
  
  uint32_t now = millis();
  uint32_t dt = now - last_update_ms_;
  if (dt < 1000 / SERVO_UPDATE_RATE) return;
  last_update_ms_ = now;
  
  for (int i = 0; i < NUM_SERVOS; i++) {
    if (!attached_[i]) continue;
    
    if (current_angles_[i] != target_angles_[i]) {
      moving_[i] = true;
      int16_t diff = (int16_t)target_angles_[i] - (int16_t)current_angles_[i];
      int16_t step = (diff > 0) ? step_size_[i] : -step_size_[i];
      
      if (abs(diff) <= abs(step)) {
        current_angles_[i] = target_angles_[i];
        moving_[i] = false;
      } else {
        current_angles_[i] += step;
      }
      
      current_pulses_[i] = angleToPulse(i, current_angles_[i]);
      servos_[i].writeMicroseconds(current_pulses_[i]);
    } else {
      moving_[i] = false;
    }
  }
}

bool ServoController::setAngle(uint8_t index, uint16_t angle_x10) {
  if (!validateIndex(index)) return false;
  if (!checkSafety(index, angle_x10, angleToPulse(index, angle_x10))) return false;
  
  target_angles_[index] = constrain(angle_x10, configs_[index].min_angle, configs_[index].max_angle);
  return true;
}

bool ServoController::setPulse(uint8_t index, uint16_t pulse_us) {
  if (!validateIndex(index)) return false;
  if (!checkSafety(index, pulseToAngle(index, pulse_us), pulse_us)) return false;
  
  uint16_t angle = pulseToAngle(index, pulse_us);
  target_angles_[index] = constrain(angle, configs_[index].min_angle, configs_[index].max_angle);
  return true;
}

bool ServoController::setAllAngles(const uint16_t* angles_x10) {
  if (!angles_x10) return false;

  for (int i = 0; i < NUM_SERVOS; i++) {
    if (!checkSafety(i, angles_x10[i], angleToPulse(i, angles_x10[i]))) return false;
  }

  for (int i = 0; i < NUM_SERVOS; i++) {
    target_angles_[i] = constrain(
      angles_x10[i], configs_[i].min_angle, configs_[i].max_angle
    );
  }
  return true;
}

uint16_t ServoController::getAngle(uint8_t index) const {
  if (!validateIndex(index)) return 0;
  return current_angles_[index];
}

uint16_t ServoController::getPulse(uint8_t index) const {
  if (!validateIndex(index)) return 0;
  return current_pulses_[index];
}

bool ServoController::isAttached(uint8_t index) const {
  if (!validateIndex(index)) return false;
  return attached_[index];
}

const ServoConfig* ServoController::getConfig(uint8_t index) const {
  if (!validateIndex(index)) return nullptr;
  return &configs_[index];
}

bool ServoController::setLimits(uint8_t index, uint16_t min_angle_x10, uint16_t max_angle_x10,
                                uint16_t min_pulse, uint16_t max_pulse, uint16_t center_pulse) {
  if (!validateIndex(index)) return false;
  if (min_angle_x10 >= max_angle_x10) return false;
  if (min_pulse >= max_pulse) return false;
  if (center_pulse < min_pulse || center_pulse > max_pulse) return false;
  if (min_pulse < 300 || max_pulse > 2700) return false;  // Hardware limits
  
  configs_[index].min_angle = min_angle_x10;
  configs_[index].max_angle = max_angle_x10;
  configs_[index].min_pulse = min_pulse;
  configs_[index].max_pulse = max_pulse;
  configs_[index].center_pulse = center_pulse;
  
  // Re-attach with new limits
  if (attached_[index]) {
    servos_[index].detach();
    servos_[index].attach(configs_[index].pin, configs_[index].min_pulse, configs_[index].max_pulse);
    attached_[index] = true;
  }
  
  // Clamp current target
  target_angles_[index] = constrain(target_angles_[index], min_angle_x10, max_angle_x10);
  current_angles_[index] = constrain(current_angles_[index], min_angle_x10, max_angle_x10);
  current_pulses_[index] = angleToPulse(index, current_angles_[index]);
  
  return true;
}

bool ServoController::setReversed(uint8_t index, bool reversed) {
  if (!validateIndex(index)) return false;
  configs_[index].reversed = reversed;
  return true;
}

void ServoController::writeEEPROMConfig() {
  uint16_t addr = EEPROM_CONFIG_START;

  // Magic number
  EEPROM.put(addr, (uint16_t)EEPROM_CONFIG_MAGIC);
  addr += 2;

  // Version
  EEPROM.put(addr, (uint16_t)1);
  addr += 2;

  // Servo configs
  for (int i = 0; i < NUM_SERVOS; i++) {
    EEPROM.put(addr, configs_[i]);
    addr += sizeof(ServoConfig);
  }
}

bool ServoController::saveConfig() {
  writeEEPROMConfig();
  return true;
}

bool ServoController::loadConfig() {
  uint16_t addr = EEPROM_CONFIG_START;
  
  uint16_t magic;
  EEPROM.get(addr, magic);
  addr += 2;
  
  if (magic != EEPROM_CONFIG_MAGIC) {
    // No valid config, use defaults
    return false;
  }
  
  uint16_t version;
  EEPROM.get(addr, version);
  addr += 2;
  
  if (version != 1) {
    return false;
  }
  
  for (int i = 0; i < NUM_SERVOS; i++) {
    EEPROM.get(addr, configs_[i]);
    addr += sizeof(ServoConfig);
    
    // Re-attach with loaded limits
    if (attached_[i]) {
      servos_[i].detach();
      servos_[i].attach(configs_[i].pin, configs_[i].min_pulse, configs_[i].max_pulse);
      attached_[i] = true;
    }
  }
  
  return true;
}

void ServoController::resetToDefaults() {
  emergency_stop_ = false;
  for (int i = 0; i < NUM_SERVOS; i++) {
    applyDefaultConfig(i);
    configs_[i].pin = servo_pins[i];
    strncpy(configs_[i].name, servo_names[i], sizeof(configs_[i].name) - 1);
    
    if (attached_[i]) {
      servos_[i].detach();
      servos_[i].attach(configs_[i].pin, configs_[i].min_pulse, configs_[i].max_pulse);
    }
    
    current_angles_[i] = pulseToAngle(i, configs_[i].center_pulse);
    target_angles_[i] = current_angles_[i];
    current_pulses_[i] = configs_[i].center_pulse;
    if (attached_[i]) {
      servos_[i].writeMicroseconds(current_pulses_[i]);
    }
  }
}

void ServoController::emergencyStop() {
  emergency_stop_ = true;
  for (int i = 0; i < NUM_SERVOS; i++) {
    if (attached_[i]) {
      target_angles_[i] = current_angles_[i];
      moving_[i] = false;
      servos_[i].writeMicroseconds(current_pulses_[i]);
    }
  }
}

bool ServoController::isEmergencyStopped() const {
  return emergency_stop_;
}

void ServoController::releaseAll() {
  for (int i = 0; i < NUM_SERVOS; i++) {
    if (attached_[i]) {
      servos_[i].detach();
      attached_[i] = false;
    }
  }
}

bool ServoController::checkSafety(uint8_t index, uint16_t angle_x10, uint16_t pulse_us) const {
  if (emergency_stop_) return false;
  
  const ServoConfig& cfg = configs_[index];
  if (angle_x10 < cfg.min_angle || angle_x10 > cfg.max_angle) return false;
  if (pulse_us < cfg.min_pulse || pulse_us > cfg.max_pulse) return false;
  
  return true;
}

void ServoController::calibrateCenter(uint8_t index) {
  if (!validateIndex(index)) return;
  configs_[index].center_pulse = current_pulses_[index];
}

void ServoController::calibrateMin(uint8_t index) {
  if (!validateIndex(index)) return;
  configs_[index].min_pulse = current_pulses_[index];
  configs_[index].min_angle = current_angles_[index];
}

void ServoController::calibrateMax(uint8_t index) {
  if (!validateIndex(index)) return;
  configs_[index].max_pulse = current_pulses_[index];
  configs_[index].max_angle = current_angles_[index];
}

uint32_t ServoController::getLastUpdateMs() const {
  return last_update_ms_;
}

bool ServoController::isMoving(uint8_t index) const {
  if (!validateIndex(index)) return false;
  return moving_[index];
}

uint16_t ServoController::angleToPulse(uint8_t index, uint16_t angle_x10) const {
  const ServoConfig& cfg = configs_[index];
  float angle = angle_x10 / 10.0f;
  float range = (cfg.max_angle - cfg.min_angle) / 10.0f;
  float pulse_range = cfg.max_pulse - cfg.min_pulse;
  
  float ratio = (angle - cfg.min_angle / 10.0f) / range;
  ratio = constrain(ratio, 0.0f, 1.0f);
  
  uint16_t pulse = cfg.min_pulse + (uint16_t)(ratio * pulse_range);
  
  if (cfg.reversed) {
    pulse = cfg.max_pulse - (pulse - cfg.min_pulse);
  }
  
  return pulse;
}

uint16_t ServoController::pulseToAngle(uint8_t index, uint16_t pulse_us) const {
  const ServoConfig& cfg = configs_[index];
  uint16_t pulse = pulse_us;
  
  if (cfg.reversed) {
    pulse = cfg.max_pulse - (pulse - cfg.min_pulse);
  }
  
  float ratio = (float)(pulse - cfg.min_pulse) / (cfg.max_pulse - cfg.min_pulse);
  ratio = constrain(ratio, 0.0f, 1.0f);
  
  float angle = cfg.min_angle / 10.0f + ratio * (cfg.max_angle - cfg.min_angle) / 10.0f;
  return (uint16_t)(angle * 10);
}

bool ServoController::validateIndex(uint8_t index) const {
  return index < NUM_SERVOS;
}

void ServoController::applyDefaultConfig(uint8_t index) {
  configs_[index].min_pulse = DEFAULT_MIN_PULSE;
  configs_[index].max_pulse = DEFAULT_MAX_PULSE;
  configs_[index].center_pulse = DEFAULT_CENTER;
  configs_[index].min_angle = DEFAULT_MIN_ANGLE * 10;
  configs_[index].max_angle = DEFAULT_MAX_ANGLE * 10;
  configs_[index].reversed = false;
  configs_[index].pin = servo_pins[index];
  strncpy(configs_[index].name, servo_names[index], sizeof(configs_[index].name) - 1);
}