#ifndef SEQUENCE_H
#define SEQUENCE_H

#include "Config.h"

// Sequence IDs for built-in motions. Custom sequences can be loaded from
// EEPROM; the firmware always falls back to these built-in definitions when
// the stored data is missing or invalid.
enum SequenceID {
  SEQ_BUILTIN_START = 0,
  SEQ_GREET = 0,        // Stand, raise arms, lower arms, return to stand.
  SEQ_SIT = 1,          // Bend knees and ankles into a sitting pose.
  SEQ_STAND = 2,        // From sit, extend legs to standing pose.
  SEQ_WAVE = 3,         // Raise right arm and wave side to side.
  SEQ_WALK_PREP = 4,    // Shift weight and shift feet to walking stance.
  SEQ_BUILTIN_END = 4
};

// Default neutral pose: all servos at center. Used as the start/end keyframe
// for every built-in sequence so motions always return the robot to a known
// safe configuration.
#define NEUTRAL_ANGLE 900  // 90.0 degrees * 10

#endif // SEQUENCE_H