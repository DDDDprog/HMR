#include "Protocol.h"
#include <Arduino.h>

Protocol::Protocol() 
  : rx_state_(RX_WAIT_START)
  , rx_index_(0)
  , rx_expected_len_(0)
  , streaming_enabled_(false)
  , stream_rate_ms_(100)
  , last_stream_ms_(0)
  , rx_count_(0)
  , tx_count_(0)
  , crc_errors_(0)
  , timeout_errors_(0) {
  memset(&rx_packet_, 0, sizeof(rx_packet_));
}

bool Protocol::encodePacket(uint8_t cmd, const uint8_t* payload, uint8_t len, uint8_t* buffer, uint16_t* out_len) {
  if (len > MAX_PAYLOAD_SIZE) return false;
  
  uint16_t idx = 0;
  buffer[idx++] = PROTOCOL_START_BYTE;
  buffer[idx++] = cmd;
  buffer[idx++] = len;
  
  if (payload && len > 0) {
    memcpy(&buffer[idx], payload, len);
    idx += len;
  }
  
  uint8_t checksum = calculateChecksum(&buffer[1], idx - 1);  // Exclude start byte
  buffer[idx++] = checksum;
  buffer[idx++] = PROTOCOL_END_BYTE;
  
  *out_len = idx;
  return true;
}

bool Protocol::decodePacket(const uint8_t* buffer, uint16_t len, Packet* packet) {
  if (len < 5) return false;  // Minimum: start, cmd, len, checksum, end
  if (buffer[0] != PROTOCOL_START_BYTE) return false;
  if (buffer[len - 1] != PROTOCOL_END_BYTE) return false;
  
  uint8_t payload_len = buffer[2];
  if (len != 5 + payload_len) return false;
  if (payload_len > MAX_PAYLOAD_SIZE) return false;
  
  packet->start = buffer[0];
  packet->cmd = buffer[1];
  packet->len = payload_len;
  memcpy(packet->payload, &buffer[3], payload_len);
  packet->checksum = buffer[3 + payload_len];
  packet->end = buffer[4 + payload_len];
  
  return verifyChecksum(packet);
}

uint8_t Protocol::calculateChecksum(const uint8_t* data, uint16_t len) {
  uint8_t sum = 0;
  for (uint16_t i = 0; i < len; i++) {
    sum ^= data[i];
  }
  return sum;
}

bool Protocol::verifyChecksum(const Packet* packet) {
  uint8_t calc = calculateChecksum(&packet->cmd, 2 + packet->len);  // cmd + len + payload
  return calc == packet->checksum;
}

void Protocol::sendAck(uint8_t cmd) {
  uint8_t payload[1] = { cmd };
  uint8_t buffer[64];
  uint16_t len;
  if (encodePacket(RSP_ACK, payload, 1, buffer, &len)) {
    ROBOT_SERIAL.write(buffer, len);
    tx_count_++;
  }
}

void Protocol::sendNack(uint8_t cmd, uint8_t error) {
  uint8_t payload[2] = { cmd, error };
  uint8_t buffer[64];
  uint16_t len;
  if (encodePacket(RSP_NACK, payload, 2, buffer, &len)) {
    ROBOT_SERIAL.write(buffer, len);
    tx_count_++;
  }
}

void Protocol::sendVersion() {
  uint8_t payload[5];
  payload[0] = FIRMWARE_VERSION[0] - '0'; // Major
  payload[1] = FIRMWARE_VERSION[2] - '0'; // Minor
  payload[2] = FIRMWARE_VERSION[4] - '0'; // Patch
  payload[3] = NUM_SERVOS;
  payload[4] = 0; // Reserved
  
  uint8_t buffer[64];
  uint16_t len;
  if (encodePacket(RSP_VERSION, payload, 5, buffer, &len)) {
    ROBOT_SERIAL.write(buffer, len);
    tx_count_++;
  }
}

void Protocol::sendServoState(uint8_t index, uint16_t angle, uint16_t pulse, bool attached) {
  uint8_t payload[6];
  payload[0] = index;
  payload[1] = (angle >> 8) & 0xFF;
  payload[2] = angle & 0xFF;
  payload[3] = (pulse >> 8) & 0xFF;
  payload[4] = pulse & 0xFF;
  payload[5] = attached ? 1 : 0;
  
  uint8_t buffer[64];
  uint16_t len;
  if (encodePacket(RSP_SERVO_STATE, payload, 6, buffer, &len)) {
    ROBOT_SERIAL.write(buffer, len);
    tx_count_++;
  }
}

void Protocol::sendAllServoStates() {
  // This will be called from main loop with actual servo data
  // Placeholder - actual implementation in main sketch
}

void Protocol::sendServoLimits(uint8_t index) {
  // Placeholder - actual implementation in main sketch
}

void Protocol::sendIMUData(const IMUData* imu) {
  uint8_t payload[54];
  payload[0] = (imu->ax >> 8) & 0xFF; payload[1] = imu->ax & 0xFF;
  payload[2] = (imu->ay >> 8) & 0xFF; payload[3] = imu->ay & 0xFF;
  payload[4] = (imu->az >> 8) & 0xFF; payload[5] = imu->az & 0xFF;
  payload[6] = (imu->gx >> 8) & 0xFF; payload[7] = imu->gx & 0xFF;
  payload[8] = (imu->gy >> 8) & 0xFF; payload[9] = imu->gy & 0xFF;
  payload[10] = (imu->gz >> 8) & 0xFF; payload[11] = imu->gz & 0xFF;
  payload[12] = (imu->temp >> 8) & 0xFF; payload[13] = imu->temp & 0xFF;
  
  // Float values as 4 bytes each (IEEE 754)
  uint8_t* fptr;
  fptr = (uint8_t*)&imu->ax_g; payload[14] = fptr[0]; payload[15] = fptr[1]; payload[16] = fptr[2]; payload[17] = fptr[3];
  fptr = (uint8_t*)&imu->ay_g; payload[18] = fptr[0]; payload[19] = fptr[1]; payload[20] = fptr[2]; payload[21] = fptr[3];
  fptr = (uint8_t*)&imu->az_g; payload[22] = fptr[0]; payload[23] = fptr[1]; payload[24] = fptr[2]; payload[25] = fptr[3];
  fptr = (uint8_t*)&imu->timestamp_ms; payload[26] = fptr[0]; payload[27] = fptr[1]; payload[28] = fptr[2]; payload[29] = fptr[3];

  const float telemetry[6] = {
    imu->gx_dps, imu->gy_dps, imu->gz_dps,
    imu->roll, imu->pitch, imu->yaw
  };
  memcpy(&payload[30], telemetry, sizeof(telemetry));
  
  uint8_t buffer[64];
  uint16_t len;
  if (encodePacket(RSP_IMU_DATA, payload, 54, buffer, &len)) {
    ROBOT_SERIAL.write(buffer, len);
    tx_count_++;
  }
}

void Protocol::sendIMUOffsets(const IMUOffsets* offsets) {
  uint8_t payload[12];
  payload[0] = (offsets->ax_off >> 8) & 0xFF; payload[1] = offsets->ax_off & 0xFF;
  payload[2] = (offsets->ay_off >> 8) & 0xFF; payload[3] = offsets->ay_off & 0xFF;
  payload[4] = (offsets->az_off >> 8) & 0xFF; payload[5] = offsets->az_off & 0xFF;
  payload[6] = (offsets->gx_off >> 8) & 0xFF; payload[7] = offsets->gx_off & 0xFF;
  payload[8] = (offsets->gy_off >> 8) & 0xFF; payload[9] = offsets->gy_off & 0xFF;
  payload[10] = (offsets->gz_off >> 8) & 0xFF; payload[11] = offsets->gz_off & 0xFF;
  
  uint8_t buffer[64];
  uint16_t len;
  if (encodePacket(RSP_IMU_OFFSETS, payload, 12, buffer, &len)) {
    ROBOT_SERIAL.write(buffer, len);
    tx_count_++;
  }
}

void Protocol::sendDiagnostics(const Diagnostics* diag) {
  uint8_t payload[60];
  uint8_t* p = payload;
  
  // Uptime (4 bytes)
  *p++ = (diag->uptime_ms >> 24) & 0xFF;
  *p++ = (diag->uptime_ms >> 16) & 0xFF;
  *p++ = (diag->uptime_ms >> 8) & 0xFF;
  *p++ = diag->uptime_ms & 0xFF;
  
  // Free RAM (2 bytes)
  *p++ = (diag->free_ram >> 8) & 0xFF;
  *p++ = diag->free_ram & 0xFF;
  
  // Servo currents (14 * 2 = 28 bytes)
  for (int i = 0; i < NUM_SERVOS; i++) {
    *p++ = (diag->servo_currents[i] >> 8) & 0xFF;
    *p++ = diag->servo_currents[i] & 0xFF;
  }
  
  // Total current (2 bytes)
  *p++ = (diag->total_current_ma >> 8) & 0xFF;
  *p++ = diag->total_current_ma & 0xFF;
  
  // Voltage (2 bytes)
  *p++ = (diag->voltage_mv >> 8) & 0xFF;
  *p++ = diag->voltage_mv & 0xFF;
  
  // Temperature (2 bytes, signed)
  *p++ = (diag->temperature_c >> 8) & 0xFF;
  *p++ = diag->temperature_c & 0xFF;
  
  // Packet counts (4 * 4 = 16 bytes)
  *p++ = (diag->packets_rx >> 24) & 0xFF; *p++ = (diag->packets_rx >> 16) & 0xFF; *p++ = (diag->packets_rx >> 8) & 0xFF; *p++ = diag->packets_rx & 0xFF;
  *p++ = (diag->packets_tx >> 24) & 0xFF; *p++ = (diag->packets_tx >> 16) & 0xFF; *p++ = (diag->packets_tx >> 8) & 0xFF; *p++ = diag->packets_tx & 0xFF;
  *p++ = (diag->crc_errors >> 24) & 0xFF; *p++ = (diag->crc_errors >> 16) & 0xFF; *p++ = (diag->crc_errors >> 8) & 0xFF; *p++ = diag->crc_errors & 0xFF;
  *p++ = (diag->timeout_errors >> 24) & 0xFF; *p++ = (diag->timeout_errors >> 16) & 0xFF; *p++ = (diag->timeout_errors >> 8) & 0xFF; *p++ = diag->timeout_errors & 0xFF;
  
  // Flags (1 byte)
  uint8_t flags = 0;
  if (diag->emergency_stop) flags |= 0x01;
  if (diag->imu_connected) flags |= 0x02;
  *p++ = flags;
  
  // Firmware version (3 bytes)
  *p++ = diag->firmware_version[0];
  *p++ = diag->firmware_version[1];
  *p++ = diag->firmware_version[2];
  
  uint16_t len = p - payload;
  uint8_t buffer[64];
  uint16_t out_len;
  if (encodePacket(RSP_DIAGNOSTICS, payload, len, buffer, &out_len)) {
    ROBOT_SERIAL.write(buffer, out_len);
    tx_count_++;
  }
}

void Protocol::sendError(uint8_t error) {
  uint8_t payload[1] = { error };
  uint8_t buffer[64];
  uint16_t len;
  if (encodePacket(RSP_ERROR, payload, 1, buffer, &len)) {
    ROBOT_SERIAL.write(buffer, len);
    tx_count_++;
  }
}

void Protocol::setStreaming(bool enabled, uint16_t rate_ms) {
  streaming_enabled_ = enabled;
  stream_rate_ms_ = rate_ms;
  last_stream_ms_ = millis();
}

bool Protocol::isStreaming() const {
  return streaming_enabled_;
}

uint16_t Protocol::getStreamRate() const {
  return stream_rate_ms_;
}

void Protocol::updateStream(uint32_t current_ms) {
  if (streaming_enabled_ && (current_ms - last_stream_ms_ >= stream_rate_ms_)) {
    last_stream_ms_ = current_ms;
    // Actual streaming handled in main loop
  }
}

void Protocol::incrementRx() { rx_count_++; }
void Protocol::incrementTx() { tx_count_++; }
void Protocol::incrementCRCErrors() { crc_errors_++; }
void Protocol::incrementTimeoutErrors() { timeout_errors_++; }

uint32_t Protocol::getRxCount() const { return rx_count_; }
uint32_t Protocol::getTxCount() const { return tx_count_; }
uint32_t Protocol::getCRCErrors() const { return crc_errors_; }
uint32_t Protocol::getTimeoutErrors() const { return timeout_errors_; }

void Protocol::resetRxBuffer() {
  rx_state_ = RX_WAIT_START;
  rx_index_ = 0;
  rx_expected_len_ = 0;
  memset(&rx_packet_, 0, sizeof(rx_packet_));
}

bool Protocol::processByte(uint8_t byte, Packet* out_packet) {
  switch (rx_state_) {
    case RX_WAIT_START:
      if (byte == PROTOCOL_START_BYTE) {
        rx_packet_.start = byte;
        rx_state_ = RX_READ_CMD;
      }
      break;
      
    case RX_READ_CMD:
      rx_packet_.cmd = byte;
      rx_state_ = RX_READ_LEN;
      break;
      
    case RX_READ_LEN:
      if (byte > MAX_PAYLOAD_SIZE) {
        rx_state_ = RX_WAIT_START;
        incrementCRCErrors();
        return false;
      }
      rx_packet_.len = byte;
      rx_expected_len_ = byte;
      rx_index_ = 0;
      if (rx_expected_len_ == 0) {
        rx_state_ = RX_READ_CHECKSUM;
      } else {
        rx_state_ = RX_READ_PAYLOAD;
      }
      break;
      
    case RX_READ_PAYLOAD:
      rx_packet_.payload[rx_index_++] = byte;
      if (rx_index_ >= rx_expected_len_) {
        rx_state_ = RX_READ_CHECKSUM;
      }
      break;
      
    case RX_READ_CHECKSUM:
      rx_packet_.checksum = byte;
      rx_state_ = RX_READ_END;
      break;
      
    case RX_READ_END:
      rx_packet_.end = byte;
      rx_state_ = RX_WAIT_START;
      
      if (verifyChecksum(&rx_packet_) && rx_packet_.end == PROTOCOL_END_BYTE) {
        *out_packet = rx_packet_;
        incrementRx();
        return true;
      } else {
        incrementCRCErrors();
        return false;
      }
  }
  return false;
}