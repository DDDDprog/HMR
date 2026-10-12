#ifndef CONFIG_H
#define CONFIG_H

// Firmware version
#define FIRMWARE_VERSION "1.1.0"
#define FIRMWARE_DATE "2026-10-12"

// Servo pin assignments (Arduino Mega 2560)
#define PIN_LEFT_HIP      2
#define PIN_LEFT_KNEE     3
#define PIN_LEFT_ANKLE    4
#define PIN_RIGHT_HIP     5
#define PIN_RIGHT_KNEE    6
#define PIN_RIGHT_ANKLE   7
#define PIN_LEFT_SHOULDER 8
#define PIN_LEFT_ELBOW    9
#define PIN_LEFT_CLAW     10
#define PIN_RIGHT_SHOULDER 11
#define PIN_RIGHT_ELBOW   12
#define PIN_RIGHT_CLAW    13
#define PIN_HEAD_PAN      22
#define PIN_HEAD_TILT     23

// Total number of servos
#define NUM_SERVOS 14

// Servo index mapping (matches GUI order)
enum ServoIndex {
  IDX_LEFT_HIP      = 0,
  IDX_LEFT_KNEE     = 1,
  IDX_LEFT_ANKLE    = 2,
  IDX_RIGHT_HIP     = 3,
  IDX_RIGHT_KNEE    = 4,
  IDX_RIGHT_ANKLE   = 5,
  IDX_LEFT_SHOULDER = 6,
  IDX_LEFT_ELBOW    = 7,
  IDX_LEFT_CLAW     = 8,
  IDX_RIGHT_SHOULDER = 9,
  IDX_RIGHT_ELBOW   = 10,
  IDX_RIGHT_CLAW    = 11,
  IDX_HEAD_PAN      = 12,
  IDX_HEAD_TILT     = 13
};

// Default servo pulse width limits (microseconds)
// Standard servos: 1000-2000us, but many work 500-2500us
// Adjust per servo during calibration
#define DEFAULT_MIN_PULSE  500
#define DEFAULT_MAX_PULSE  2500
#define DEFAULT_CENTER     1500

// Default angle limits (degrees)
// Mechanical limits - adjust per robot during calibration
#define DEFAULT_MIN_ANGLE  0
#define DEFAULT_MAX_ANGLE  180

// Servo update rate (Hz)
#define SERVO_UPDATE_RATE 50

// Serial communication
#define SERIAL_BAUD_RATE 115200
#define SERIAL_TIMEOUT_MS 100

// Runtime serial transport: 0 = wired USB Serial, 1 = Bluetooth Serial1.
// Match this setting to the GUI link profile, then reflash the Mega.
#define USE_BLUETOOTH_SERIAL 0

#if USE_BLUETOOTH_SERIAL
  #define ROBOT_SERIAL Serial1
  #define ROBOT_BAUD_RATE 9600
#else
  #define ROBOT_SERIAL Serial
  #define ROBOT_BAUD_RATE SERIAL_BAUD_RATE
#endif

// Protocol
#define PROTOCOL_START_BYTE 0xAA
#define PROTOCOL_END_BYTE   0x55
#define MAX_PAYLOAD_SIZE    64

// Command IDs
enum CommandID {
  CMD_PING              = 0x01,
  CMD_GET_VERSION       = 0x02,
  CMD_SET_SERVO_ANGLE   = 0x10,
  CMD_SET_SERVO_PULSE   = 0x11,
  CMD_GET_SERVO_STATE   = 0x12,
  CMD_SET_ALL_SERVOS    = 0x13,
  CMD_SET_SERVO_LIMITS  = 0x20,
  CMD_GET_SERVO_LIMITS  = 0x21,
  CMD_SAVE_CONFIG       = 0x22,
  CMD_LOAD_CONFIG       = 0x23,
  CMD_RESET_SERVOS      = 0x30,
  CMD_EMERGENCY_STOP    = 0x31,
  CMD_GET_IMU_DATA      = 0x40,
  CMD_CALIBRATE_IMU     = 0x41,
  CMD_SET_IMU_OFFSETS   = 0x42,
  CMD_GET_IMU_OFFSETS   = 0x43,
  CMD_START_STREAM      = 0x50,
  CMD_STOP_STREAM       = 0x51,
  CMD_SET_STREAM_RATE   = 0x52,
  CMD_RUN_SEQUENCE      = 0x60,
  CMD_STOP_SEQUENCE     = 0x61,
  CMD_GET_SEQUENCE_STATE = 0x62,
  CMD_GET_DIAGNOSTICS   = 0x70
};

// Response IDs
enum ResponseID {
  RSP_ACK               = 0x80,
  RSP_NACK              = 0x81,
  RSP_VERSION           = 0x82,
  RSP_SERVO_STATE       = 0x90,
  RSP_ALL_SERVO_STATE   = 0x91,
  RSP_SERVO_LIMITS      = 0x92,
  RSP_IMU_DATA          = 0xA0,
  RSP_IMU_OFFSETS       = 0xA1,
  RSP_DIAGNOSTICS       = 0xB0,
  RSP_ERROR             = 0xFF
};

// Error codes
enum ErrorCode {
  ERR_NONE              = 0x00,
  ERR_INVALID_CMD       = 0x01,
  ERR_INVALID_PARAM     = 0x02,
  ERR_CHECKSUM_FAIL     = 0x03,
  ERR_SERVO_INDEX       = 0x04,
  ERR_ANGLE_OUT_OF_RANGE = 0x05,
  ERR_PULSE_OUT_OF_RANGE = 0x06,
  ERR_IMU_NOT_FOUND     = 0x07,
  ERR_IMU_INIT_FAIL     = 0x08,
  ERR_EEPROM_WRITE_FAIL = 0x09,
  ERR_SAFETY_VIOLATION  = 0x0A,
  ERR_COMM_TIMEOUT      = 0x0B,
  ERR_SEQUENCE_RUNNING  = 0x0C
};

// IMU (MPU-6050) I2C address
#define MPU6050_ADDR 0x68

// IMU sample rate divider (1kHz / (1 + divider))
#define IMU_SAMPLE_RATE_DIVIDER 9  // 100 Hz

// IMU filter settings
#define IMU_DLPF_CFG 3  // 44 Hz bandwidth

// Safety monitor
#define MAX_SERVO_CURRENT_MA 2000  // Per servo (adjust for your servos)
#define MAX_TOTAL_CURRENT_MA 8000  // Total for all servos
#define VOLTAGE_MONITOR_PIN A0     // Optional: voltage divider for battery monitoring
#define MIN_SAFE_VOLTAGE_MV 4500   // 4.5V minimum
#define MAX_SAFE_VOLTAGE_MV 6500   // 6.5V maximum

// Temperature monitoring (optional, requires sensor)
// #define TEMP_MONITOR_PIN A1
// #define MAX_SAFE_TEMP_C 70

// Watchdog timeout (ms)
#define WATCHDOG_TIMEOUT_MS 5000

// EEPROM addresses for persistent config
#define EEPROM_CONFIG_START 0
#define EEPROM_CONFIG_MAGIC 0x4852  // "HR" in hex

// Motion sequence support
// A sequence is a series of keyframes. Each keyframe sets all 14 servo angles
// (angle * 10, big-endian u16 each = 28 bytes) plus a duration in milliseconds
// (u16). The firmware interpolates each servo from the previous keyframe to the
// next over the keyframe duration, then advances. Sequences are stored in
// EEPROM after the servo config block and can be triggered over the serial
// protocol. Built-in sequences are defined in Sequence.h.
#define MAX_KEYFRAMES 16
#define MAX_SEQUENCES 8
#define SEQUENCE_EE_START (EEPROM_CONFIG_START + 2 + 2 + NUM_SERVOS * sizeof(ServoConfig))

struct Keyframe {
  uint16_t angles[NUM_SERVOS];  // angle * 10
  uint16_t duration_ms;         // time to hold this keyframe
};

struct MotionSequence {
  uint8_t id;
  char name[24];
  uint8_t keyframe_count;
  Keyframe keyframes[MAX_KEYFRAMES];
  bool loop;
};

// Debug
// #define DEBUG_SERIAL 1

#endif // CONFIG_H