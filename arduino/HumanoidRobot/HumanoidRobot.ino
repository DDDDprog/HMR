#include "Config.h"
#include "Protocol.h"
#include "ServoController.h"
#include "IMUHandler.h"

ServoController servos;
IMUHandler imu;
Protocol protocol;

uint32_t last_diag_ms = 0;
uint32_t last_heartbeat_ms = 0;
bool streaming_imu = false;
bool streaming_servos = false;

void setup() {
  ROBOT_SERIAL.begin(ROBOT_BAUD_RATE);
  #if !USE_BLUETOOTH_SERIAL
    while (!Serial && millis() < 2000);  // Wait for serial (max 2s)
  #else
    delay(1000);  // Bluetooth modules need a short delay after power-on
  #endif

  // Initialize servos
  servos.begin();
  
  // Initialize IMU
  imu.begin();
  
  // Send startup version
  protocol.sendVersion();
  
  last_diag_ms = millis();
  last_heartbeat_ms = millis();
}

void loop() {
  uint32_t now = millis();
  
  // Process serial commands
  processSerial();
  
  // Update servos
  servos.update();
  
  // Update IMU
  if (imu.isConnected()) {
    imu.update();
  }
  
  // Stream IMU data
  if (streaming_imu && imu.isConnected()) {
    if (now - last_heartbeat_ms >= protocol.getStreamRate()) {
      protocol.sendIMUData(imu.getData());
      last_heartbeat_ms = now;
    }
  }
  
  // Stream servo states
  if (streaming_servos) {
    if (now - last_heartbeat_ms >= protocol.getStreamRate()) {
      sendAllServoStates();
      last_heartbeat_ms = now;
    }
  }
  
  // Periodic diagnostics
  if (now - last_diag_ms >= 5000) {
    sendDiagnostics();
    last_diag_ms = now;
  }
  
  // Watchdog: if no commands received for WATCHDOG_TIMEOUT_MS, emergency stop
  // (Handled in processSerial timeout)
}

void processSerial() {
  static uint32_t last_rx_ms = 0;
  
  while (ROBOT_SERIAL.available()) {
    uint8_t byte = ROBOT_SERIAL.read();
    Packet packet;
    
    if (protocol.processByte(byte, &packet)) {
      last_rx_ms = millis();
      handleCommand(packet);
    }
  }
  
  // Watchdog check
  if (millis() - last_rx_ms > WATCHDOG_TIMEOUT_MS && last_rx_ms > 0) {
    servos.emergencyStop();
    protocol.sendError(ERR_COMM_TIMEOUT);
    last_rx_ms = millis();  // Prevent repeated triggers
  }
}

void handleCommand(const Packet& packet) {
  switch (packet.cmd) {
    case CMD_PING:
      protocol.sendAck(CMD_PING);
      break;
      
    case CMD_GET_VERSION:
      protocol.sendVersion();
      break;
      
    case CMD_SET_SERVO_ANGLE:
      if (packet.len == 3) {
        uint8_t index = packet.payload[0];
        uint16_t angle = (packet.payload[1] << 8) | packet.payload[2];
        if (servos.setAngle(index, angle)) {
          protocol.sendAck(CMD_SET_SERVO_ANGLE);
        } else {
          protocol.sendNack(CMD_SET_SERVO_ANGLE, ERR_SERVO_INDEX);
        }
      } else {
        protocol.sendNack(CMD_SET_SERVO_ANGLE, ERR_INVALID_PARAM);
      }
      break;
      
    case CMD_SET_SERVO_PULSE:
      if (packet.len == 3) {
        uint8_t index = packet.payload[0];
        uint16_t pulse = (packet.payload[1] << 8) | packet.payload[2];
        if (servos.setPulse(index, pulse)) {
          protocol.sendAck(CMD_SET_SERVO_PULSE);
        } else {
          protocol.sendNack(CMD_SET_SERVO_PULSE, ERR_SERVO_INDEX);
        }
      } else {
        protocol.sendNack(CMD_SET_SERVO_PULSE, ERR_INVALID_PARAM);
      }
      break;
      
    case CMD_GET_SERVO_STATE:
      if (packet.len == 1) {
        uint8_t index = packet.payload[0];
        if (index < NUM_SERVOS) {
          protocol.sendServoState(index, servos.getAngle(index), servos.getPulse(index), servos.isAttached(index));
        } else {
          protocol.sendNack(CMD_GET_SERVO_STATE, ERR_SERVO_INDEX);
        }
      } else {
        protocol.sendNack(CMD_GET_SERVO_STATE, ERR_INVALID_PARAM);
      }
      break;
      
    case CMD_SET_ALL_SERVOS:
      if (packet.len == NUM_SERVOS * 2) {
        uint16_t angles[NUM_SERVOS];
        for (int i = 0; i < NUM_SERVOS; i++) {
          angles[i] = (packet.payload[i * 2] << 8) | packet.payload[i * 2 + 1];
        }
        if (servos.setAllAngles(angles)) {
          protocol.sendAck(CMD_SET_ALL_SERVOS);
        } else {
          protocol.sendNack(CMD_SET_ALL_SERVOS, ERR_SAFETY_VIOLATION);
        }
      } else {
        protocol.sendNack(CMD_SET_ALL_SERVOS, ERR_INVALID_PARAM);
      }
      break;
      
    case CMD_SET_SERVO_LIMITS:
      if (packet.len == 11) {
        uint8_t index = packet.payload[0];
        uint16_t min_angle = (packet.payload[1] << 8) | packet.payload[2];
        uint16_t max_angle = (packet.payload[3] << 8) | packet.payload[4];
        uint16_t min_pulse = (packet.payload[5] << 8) | packet.payload[6];
        uint16_t max_pulse = (packet.payload[7] << 8) | packet.payload[8];
        uint16_t center_pulse = (packet.payload[9] << 8) | packet.payload[10];
        
        if (servos.setLimits(index, min_angle, max_angle, min_pulse, max_pulse, center_pulse)) {
          protocol.sendAck(CMD_SET_SERVO_LIMITS);
        } else {
          protocol.sendNack(CMD_SET_SERVO_LIMITS, ERR_INVALID_PARAM);
        }
      } else {
        protocol.sendNack(CMD_SET_SERVO_LIMITS, ERR_INVALID_PARAM);
      }
      break;
      
    case CMD_GET_SERVO_LIMITS:
      if (packet.len == 1) {
        uint8_t index = packet.payload[0];
        if (index < NUM_SERVOS) {
          sendServoLimits(index);
        } else {
          protocol.sendNack(CMD_GET_SERVO_LIMITS, ERR_SERVO_INDEX);
        }
      } else {
        protocol.sendNack(CMD_GET_SERVO_LIMITS, ERR_INVALID_PARAM);
      }
      break;
      
    case CMD_SAVE_CONFIG:
      if (servos.saveConfig()) {
        protocol.sendAck(CMD_SAVE_CONFIG);
      } else {
        protocol.sendNack(CMD_SAVE_CONFIG, ERR_EEPROM_WRITE_FAIL);
      }
      break;
      
    case CMD_LOAD_CONFIG:
      if (servos.loadConfig()) {
        protocol.sendAck(CMD_LOAD_CONFIG);
      } else {
        protocol.sendNack(CMD_LOAD_CONFIG, ERR_EEPROM_WRITE_FAIL);
      }
      break;
      
    case CMD_RESET_SERVOS:
      servos.resetToDefaults();
      protocol.sendAck(CMD_RESET_SERVOS);
      break;
      
    case CMD_EMERGENCY_STOP:
      servos.emergencyStop();
      protocol.sendAck(CMD_EMERGENCY_STOP);
      break;
      
    case CMD_GET_IMU_DATA:
      if (imu.isConnected()) {
        protocol.sendIMUData(imu.getData());
      } else {
        protocol.sendNack(CMD_GET_IMU_DATA, ERR_IMU_NOT_FOUND);
      }
      break;
      
    case CMD_CALIBRATE_IMU:
      if (imu.calibrate(1000)) {
        protocol.sendAck(CMD_CALIBRATE_IMU);
      } else {
        protocol.sendNack(CMD_CALIBRATE_IMU, ERR_IMU_INIT_FAIL);
      }
      break;
      
    case CMD_SET_IMU_OFFSETS:
      if (packet.len == 12) {
        IMUOffsets offsets;
        offsets.ax_off = (packet.payload[0] << 8) | packet.payload[1];
        offsets.ay_off = (packet.payload[2] << 8) | packet.payload[3];
        offsets.az_off = (packet.payload[4] << 8) | packet.payload[5];
        offsets.gx_off = (packet.payload[6] << 8) | packet.payload[7];
        offsets.gy_off = (packet.payload[8] << 8) | packet.payload[9];
        offsets.gz_off = (packet.payload[10] << 8) | packet.payload[11];
        offsets.valid = true;
        
        if (imu.setOffsets(&offsets)) {
          protocol.sendAck(CMD_SET_IMU_OFFSETS);
        } else {
          protocol.sendNack(CMD_SET_IMU_OFFSETS, ERR_EEPROM_WRITE_FAIL);
        }
      } else {
        protocol.sendNack(CMD_SET_IMU_OFFSETS, ERR_INVALID_PARAM);
      }
      break;
      
    case CMD_GET_IMU_OFFSETS:
      protocol.sendIMUOffsets(imu.getOffsets());
      break;
      
    case CMD_START_STREAM:
      if (packet.len >= 1) {
        uint8_t stream_type = packet.payload[0];
        uint16_t rate = 100;
        if (packet.len >= 3) {
          rate = (packet.payload[1] << 8) | packet.payload[2];
        }
        
        if (stream_type == 0x01) {  // IMU
          streaming_imu = true;
          protocol.setStreaming(true, rate);
          protocol.sendAck(CMD_START_STREAM);
        } else if (stream_type == 0x02) {  // Servos
          streaming_servos = true;
          protocol.setStreaming(true, rate);
          protocol.sendAck(CMD_START_STREAM);
        } else {
          protocol.sendNack(CMD_START_STREAM, ERR_INVALID_PARAM);
        }
      } else {
        protocol.sendNack(CMD_START_STREAM, ERR_INVALID_PARAM);
      }
      break;
      
    case CMD_STOP_STREAM:
      streaming_imu = false;
      streaming_servos = false;
      protocol.setStreaming(false);
      protocol.sendAck(CMD_STOP_STREAM);
      break;
      
    case CMD_SET_STREAM_RATE:
      if (packet.len == 2) {
        uint16_t rate = (packet.payload[0] << 8) | packet.payload[1];
        protocol.setStreaming(protocol.isStreaming(), rate);
        protocol.sendAck(CMD_SET_STREAM_RATE);
      } else {
        protocol.sendNack(CMD_SET_STREAM_RATE, ERR_INVALID_PARAM);
      }
      break;
      
    case CMD_RUN_SEQUENCE:
      // Not implemented yet - placeholder for motion sequences
      protocol.sendNack(CMD_RUN_SEQUENCE, ERR_INVALID_CMD);
      break;
      
    case CMD_STOP_SEQUENCE:
      protocol.sendAck(CMD_STOP_SEQUENCE);
      break;
      
    case CMD_GET_DIAGNOSTICS:
      sendDiagnostics();
      break;
      
    default:
      protocol.sendNack(packet.cmd, ERR_INVALID_CMD);
      break;
  }
}

void sendAllServoStates() {
  for (int i = 0; i < NUM_SERVOS; i++) {
    protocol.sendServoState(i, servos.getAngle(i), servos.getPulse(i), servos.isAttached(i));
    delayMicroseconds(100);  // Small delay to avoid buffer overflow
  }
}

void sendServoLimits(uint8_t index) {
  const ServoConfig* cfg = servos.getConfig(index);
  if (!cfg) return;
  
  uint8_t payload[11];
  payload[0] = index;
  payload[1] = (cfg->min_angle >> 8) & 0xFF; payload[2] = cfg->min_angle & 0xFF;
  payload[3] = (cfg->max_angle >> 8) & 0xFF; payload[4] = cfg->max_angle & 0xFF;
  payload[5] = (cfg->min_pulse >> 8) & 0xFF; payload[6] = cfg->min_pulse & 0xFF;
  payload[7] = (cfg->max_pulse >> 8) & 0xFF; payload[8] = cfg->max_pulse & 0xFF;
  payload[9] = (cfg->center_pulse >> 8) & 0xFF; payload[10] = cfg->center_pulse & 0xFF;
  
  uint8_t buffer[64];
  uint16_t len;
  if (protocol.encodePacket(RSP_SERVO_LIMITS, payload, 11, buffer, &len)) {
    ROBOT_SERIAL.write(buffer, len);
  }
}

void sendDiagnostics() {
  Diagnostics diag;
  diag.uptime_ms = millis();
  diag.free_ram = freeRam();
  
  for (int i = 0; i < NUM_SERVOS; i++) {
    diag.servo_currents[i] = estimateServoCurrent(i);
  }
  
  diag.total_current_ma = 0;
  for (int i = 0; i < NUM_SERVOS; i++) {
    diag.total_current_ma += diag.servo_currents[i];
  }
  
  diag.voltage_mv = readVoltage();
  diag.temperature_c = (int16_t)imu.getTemperatureC();
  diag.packets_rx = protocol.getRxCount();
  diag.packets_tx = protocol.getTxCount();
  diag.crc_errors = protocol.getCRCErrors();
  diag.timeout_errors = protocol.getTimeoutErrors();
  diag.emergency_stop = servos.isEmergencyStopped();
  diag.imu_connected = imu.isConnected();
  diag.firmware_version[0] = FIRMWARE_VERSION[0] - '0';
  diag.firmware_version[1] = FIRMWARE_VERSION[2] - '0';
  diag.firmware_version[2] = FIRMWARE_VERSION[4] - '0';
  
  protocol.sendDiagnostics(&diag);
}

int freeRam() {
  extern int __heap_start, *__brkval;
  int v;
  return (int)&v - (__brkval == 0 ? (int)&__heap_start : (int)__brkval);
}

uint16_t estimateServoCurrent(uint8_t index) {
  // Rough estimation based on movement
  // In reality, you'd need current sensors
  const ServoConfig* cfg = servos.getConfig(index);
  if (!cfg || !servos.isAttached(index)) return 0;
  
  uint16_t pulse = servos.getPulse(index);
  uint16_t center = cfg->center_pulse;
  uint16_t diff = (pulse > center) ? pulse - center : center - pulse;
  
  // Rough estimate: ~10mA at center, up to ~1000mA at extremes
  return 10 + (diff * 1000 / 1000);
}

uint16_t readVoltage() {
  // Read from voltage divider on A0
  // Assumes 100k/10k divider for 0-12V range
  // 5V reference, 10-bit ADC
  int raw = analogRead(VOLTAGE_MONITOR_PIN);
  float voltage = raw * (5.0 / 1023.0) * 11.0;  // 11x divider ratio
  return (uint16_t)(voltage * 1000);  // mV
}