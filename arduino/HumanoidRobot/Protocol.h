#ifndef PROTOCOL_H
#define PROTOCOL_H

#include "Config.h"
#include "IMUHandler.h"
#include <Arduino.h>

struct Packet {
  uint8_t start;
  uint8_t cmd;
  uint8_t len;
  uint8_t payload[MAX_PAYLOAD_SIZE];
  uint8_t checksum;
  uint8_t end;
};

struct ServoState {
  uint8_t index;
  uint16_t angle;      // 0-180 degrees * 10 (0.1 deg resolution)
  uint16_t pulse_us;   // microseconds
  bool attached;
};

struct ServoLimits {
  uint8_t index;
  uint16_t min_angle;  // *10
  uint16_t max_angle;  // *10
  uint16_t min_pulse;
  uint16_t max_pulse;
  uint16_t center_pulse;
};

struct Diagnostics {
  uint32_t uptime_ms;
  uint16_t free_ram;
  uint16_t servo_currents[NUM_SERVOS];  // mA (estimated)
  uint16_t total_current_ma;
  uint16_t voltage_mv;
  int16_t temperature_c;
  uint32_t packets_rx;
  uint32_t packets_tx;
  uint32_t crc_errors;
  uint32_t timeout_errors;
  bool emergency_stop;
  bool imu_connected;
  uint8_t firmware_version[3];
};

class Protocol {
public:
  Protocol();
  
  // Packet encoding/decoding
  bool encodePacket(uint8_t cmd, const uint8_t* payload, uint8_t len, uint8_t* buffer, uint16_t* out_len);
  bool decodePacket(const uint8_t* buffer, uint16_t len, Packet* packet);
  
  // Checksum
  uint8_t calculateChecksum(const uint8_t* data, uint16_t len);
  bool verifyChecksum(const Packet* packet);
  
  // Response helpers
  void sendAck(uint8_t cmd);
  void sendNack(uint8_t cmd, uint8_t error);
  void sendVersion();
  void sendServoState(uint8_t index, uint16_t angle, uint16_t pulse, bool attached);
  void sendAllServoStates();
  void sendServoLimits(uint8_t index);
  void sendIMUData(const IMUData* imu);
  void sendIMUOffsets(const IMUOffsets* offsets);
  void sendDiagnostics(const Diagnostics* diag);
  void sendError(uint8_t error);
  
  // Stream control
  void setStreaming(bool enabled, uint16_t rate_ms = 100);
  bool isStreaming() const;
  uint16_t getStreamRate() const;
  void updateStream(uint32_t current_ms);
  
  // Statistics
  void incrementRx();
  void incrementTx();
  void incrementCRCErrors();
  void incrementTimeoutErrors();
  uint32_t getRxCount() const;
  uint32_t getTxCount() const;
  uint32_t getCRCErrors() const;
  uint32_t getTimeoutErrors() const;
  
private:
  enum RxState {
    RX_WAIT_START,
    RX_READ_CMD,
    RX_READ_LEN,
    RX_READ_PAYLOAD,
    RX_READ_CHECKSUM,
    RX_READ_END
  };
  
  RxState rx_state_;
  uint16_t rx_index_;
  uint8_t rx_expected_len_;
  Packet rx_packet_;
  
  bool streaming_enabled_;
  uint16_t stream_rate_ms_;
  uint32_t last_stream_ms_;
  
  uint32_t rx_count_;
  uint32_t tx_count_;
  uint32_t crc_errors_;
  uint32_t timeout_errors_;
};

#endif // PROTOCOL_H