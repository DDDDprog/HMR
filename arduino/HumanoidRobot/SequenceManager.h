#ifndef SEQUENCE_MANAGER_H
#define SEQUENCE_MANAGER_H

#include "Config.h"
#include "ServoController.h"
#include <Arduino.h>
#include <EEPROM.h>

// Motion sequence runtime state.
struct SequenceState {
  uint8_t sequence_id;
  uint8_t current_keyframe;
  uint8_t total_keyframes;
  uint16_t elapsed_ms;
  uint16_t keyframe_duration_ms;
  bool running;
};

// A single keyframe: all 14 servo targets (angle * 10) plus the time to reach it.
struct Keyframe {
  const int16_t* angles;  // pointer to program-memory table, NUM_SERVOS entries
  uint16_t duration_ms;
};

// A named motion sequence composed of keyframes.
struct MotionSequence {
  uint8_t id;
  const char* name;
  const Keyframe* keyframes;
  uint8_t keyframe_count;
  bool loop;
};

class SequenceManager {
public:
  SequenceManager(ServoController* servos);

  bool begin();
  void update();

  // Sequence table access
  uint8_t getSequenceCount() const;
  const MotionSequence* getSequence(uint8_t index) const;
  const MotionSequence* getSequenceByID(uint8_t id) const;
  bool getSequenceName(uint8_t index, char* out_name, uint8_t max_len) const;

  // Execution control
  bool startSequence(uint8_t sequence_id);
  bool stopSequence();
  bool isRunning() const;
  const SequenceState& getState() const;

  // EEPROM persistence for custom sequences
  bool saveSequences();
  bool loadSequences();

  // Emergency stop integration
  void emergencyStop();

private:
  ServoController* servos_;
  SequenceState state_;

  // EEPROM layout for custom sequences
  static const uint16_t EE_ADDR = EEPROM_CONFIG_START + 2 + 2 + NUM_SERVOS * sizeof(ServoConfig);
  static const uint16_t EE_MAGIC = 0x5345;  // "SE"
  static const uint8_t EE_VERSION = 1;

  void interpolateKeyframes();
  bool validateSequence(const MotionSequence* seq);
};

#endif // SEQUENCE_MANAGER_H