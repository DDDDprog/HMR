#include "IMUHandler.h"
#include <Arduino.h>
#include <EEPROM.h>

// MPU-6050 Registers
#define MPU6050_SELF_TEST_X   0x0D
#define MPU6050_SELF_TEST_Y   0x0E
#define MPU6050_SELF_TEST_Z   0x0F
#define MPU6050_SELF_TEST_A   0x10
#define MPU6050_SMPLRT_DIV    0x19
#define MPU6050_CONFIG        0x1A
#define MPU6050_GYRO_CONFIG   0x1B
#define MPU6050_ACCEL_CONFIG  0x1C
#define MPU6050_FIFO_EN       0x23
#define MPU6050_I2C_MST_CTRL  0x24
#define MPU6050_I2C_SLV0_ADDR 0x25
#define MPU6050_I2C_SLV0_REG  0x26
#define MPU6050_I2C_SLV0_CTRL 0x27
#define MPU6050_I2C_SLV1_ADDR 0x28
#define MPU6050_I2C_SLV1_REG  0x29
#define MPU6050_I2C_SLV1_CTRL 0x2A
#define MPU6050_I2C_SLV2_ADDR 0x2B
#define MPU6050_I2C_SLV2_REG  0x2C
#define MPU6050_I2C_SLV2_CTRL 0x2D
#define MPU6050_I2C_SLV3_ADDR 0x2E
#define MPU6050_I2C_SLV3_REG  0x2F
#define MPU6050_I2C_SLV3_CTRL 0x30
#define MPU6050_I2C_SLV4_ADDR 0x31
#define MPU6050_I2C_SLV4_REG  0x32
#define MPU6050_I2C_SLV4_DO   0x33
#define MPU6050_I2C_SLV4_CTRL 0x34
#define MPU6050_I2C_SLV4_DI   0x35
#define MPU6050_I2C_MST_STATUS 0x36
#define MPU6050_INT_PIN_CFG   0x37
#define MPU6050_INT_ENABLE    0x38
#define MPU6050_INT_STATUS    0x3A
#define MPU6050_ACCEL_XOUT_H  0x3B
#define MPU6050_ACCEL_XOUT_L  0x3C
#define MPU6050_ACCEL_YOUT_H  0x3D
#define MPU6050_ACCEL_YOUT_L  0x3E
#define MPU6050_ACCEL_ZOUT_H  0x3F
#define MPU6050_ACCEL_ZOUT_L  0x40
#define MPU6050_TEMP_OUT_H    0x41
#define MPU6050_TEMP_OUT_L    0x42
#define MPU6050_GYRO_XOUT_H   0x43
#define MPU6050_GYRO_XOUT_L   0x44
#define MPU6050_GYRO_YOUT_H   0x45
#define MPU6050_GYRO_YOUT_L   0x46
#define MPU6050_GYRO_ZOUT_H   0x47
#define MPU6050_GYRO_ZOUT_L   0x48
#define MPU6050_EXT_SENS_DATA_00 0x49
#define MPU6050_USER_CTRL     0x6A
#define MPU6050_PWR_MGMT_1    0x6B
#define MPU6050_PWR_MGMT_2    0x6C
#define MPU6050_FIFO_COUNTH   0x72
#define MPU6050_FIFO_COUNTL   0x73
#define MPU6050_FIFO_R_W      0x74
#define MPU6050_WHO_AM_I      0x75

// EEPROM address for IMU offsets
#define EEPROM_IMU_OFFSETS_ADDR (EEPROM_CONFIG_START + 2 + 2 + NUM_SERVOS * sizeof(ServoConfig))

IMUHandler::IMUHandler() 
  : connected_(false)
  , sample_count_(0)
  , error_count_(0)
  , last_update_ms_(0)
  , roll_(0), pitch_(0), yaw_(0)
  , last_orientation_ms_(0)
  , accel_range_(0)
  , gyro_range_(0)
  , accel_scale_(16384.0f)  // 2g default
  , gyro_scale_(131.0f) {   // 250 dps default
  memset(&data_, 0, sizeof(data_));
  memset(&offsets_, 0, sizeof(offsets_));
  offsets_.valid = false;
}

bool IMUHandler::begin() {
  Wire.begin();
  Wire.setClock(400000);  // 400kHz I2C
  
  delay(100);  // Wait for sensor startup
  
  // Check WHO_AM_I
  uint8_t whoami;
  if (!readRegister(MPU6050_WHO_AM_I, &whoami) || whoami != 0x68) {
    connected_ = false;
    return false;
  }
  
  // Wake up (clear sleep bit)
  if (!writeRegister(MPU6050_PWR_MGMT_1, 0x00)) {
    connected_ = false;
    return false;
  }
  delay(100);
  
  // Set sample rate
  setSampleRate(IMU_SAMPLE_RATE_DIVIDER);
  
  // Set DLPF
  writeRegister(MPU6050_CONFIG, IMU_DLPF_CFG & 0x07);
  
  // Set default ranges
  setAccelRange(0);  // +/- 2g
  setGyroRange(0);   // +/- 250 dps
  
  // Disable FIFO, I2C master
  writeRegister(MPU6050_FIFO_EN, 0x00);
  writeRegister(MPU6050_USER_CTRL, 0x00);
  writeRegister(MPU6050_I2C_MST_CTRL, 0x00);
  
  // Load saved offsets
  loadOffsets();
  
  connected_ = true;
  last_update_ms_ = millis();
  last_orientation_ms_ = millis();
  
  return true;
}

bool IMUHandler::update() {
  if (!connected_) return false;
  
  uint8_t buffer[14];
  if (!readRegisters(MPU6050_ACCEL_XOUT_H, buffer, 14)) {
    error_count_++;
    return false;
  }
  
  data_.ax = (int16_t)((buffer[0] << 8) | buffer[1]);
  data_.ay = (int16_t)((buffer[2] << 8) | buffer[3]);
  data_.az = (int16_t)((buffer[4] << 8) | buffer[5]);
  data_.temp = (int16_t)((buffer[6] << 8) | buffer[7]);
  data_.gx = (int16_t)((buffer[8] << 8) | buffer[9]);
  data_.gy = (int16_t)((buffer[10] << 8) | buffer[11]);
  data_.gz = (int16_t)((buffer[12] << 8) | buffer[13]);
  
  // Apply offsets
  data_.ax -= offsets_.ax_off;
  data_.ay -= offsets_.ay_off;
  data_.az -= offsets_.az_off;
  data_.gx -= offsets_.gx_off;
  data_.gy -= offsets_.gy_off;
  data_.gz -= offsets_.gz_off;
  
  // Convert to physical units
  data_.ax_g = accelToG(data_.ax);
  data_.ay_g = accelToG(data_.ay);
  data_.az_g = accelToG(data_.az);
  data_.gx_dps = gyroToDPS(data_.gx);
  data_.gy_dps = gyroToDPS(data_.gy);
  data_.gz_dps = gyroToDPS(data_.gz);
  data_.temp_c = tempToC(data_.temp);
  
  data_.timestamp_ms = millis();
  sample_count_++;
  
  // Compute orientation
  computeOrientation();
  
  return true;
}

bool IMUHandler::isConnected() const {
  return connected_;
}

const IMUData* IMUHandler::getData() const {
  return &data_;
}

bool IMUHandler::readRaw(int16_t* ax, int16_t* ay, int16_t* az, int16_t* gx, int16_t* gy, int16_t* gz, int16_t* temp) {
  if (!connected_) return false;
  
  uint8_t buffer[14];
  if (!readRegisters(MPU6050_ACCEL_XOUT_H, buffer, 14)) {
    return false;
  }
  
  *ax = (int16_t)((buffer[0] << 8) | buffer[1]);
  *ay = (int16_t)((buffer[2] << 8) | buffer[3]);
  *az = (int16_t)((buffer[4] << 8) | buffer[5]);
  *temp = (int16_t)((buffer[6] << 8) | buffer[7]);
  *gx = (int16_t)((buffer[8] << 8) | buffer[9]);
  *gy = (int16_t)((buffer[10] << 8) | buffer[11]);
  *gz = (int16_t)((buffer[12] << 8) | buffer[13]);
  
  return true;
}

bool IMUHandler::calibrate(uint16_t samples) {
  if (!connected_) return false;
  
  int32_t ax_sum = 0, ay_sum = 0, az_sum = 0;
  int32_t gx_sum = 0, gy_sum = 0, gz_sum = 0;
  
  for (uint16_t i = 0; i < samples; i++) {
    int16_t ax, ay, az, gx, gy, gz, temp;
    if (readRaw(&ax, &ay, &az, &gx, &gy, &gz, &temp)) {
      ax_sum += ax;
      ay_sum += ay;
      az_sum += az;
      gx_sum += gx;
      gy_sum += gy;
      gz_sum += gz;
    }
    delay(2);  // ~500Hz
  }
  
  offsets_.ax_off = ax_sum / samples;
  offsets_.ay_off = ay_sum / samples;
  offsets_.az_off = az_sum / samples - (int16_t)(1.0f / accelToG(1));  // Remove gravity from Z
  offsets_.gx_off = gx_sum / samples;
  offsets_.gy_off = gy_sum / samples;
  offsets_.gz_off = gz_sum / samples;
  offsets_.valid = true;
  
  return saveOffsets();
}

bool IMUHandler::setOffsets(const IMUOffsets* offsets) {
  if (!offsets) return false;
  offsets_ = *offsets;
  return saveOffsets();
}

const IMUOffsets* IMUHandler::getOffsets() const {
  return &offsets_;
}

bool IMUHandler::saveOffsets() {
  uint16_t addr = EEPROM_IMU_OFFSETS_ADDR;
  EEPROM.put(addr, offsets_);
  return true;
}

bool IMUHandler::loadOffsets() {
  uint16_t addr = EEPROM_IMU_OFFSETS_ADDR;
  EEPROM.get(addr, offsets_);
  return offsets_.valid;
}

void IMUHandler::setSampleRate(uint8_t divider) {
  writeRegister(MPU6050_SMPLRT_DIV, divider);
}

void IMUHandler::setDLPF(uint8_t config) {
  writeRegister(MPU6050_CONFIG, config & 0x07);
}

void IMUHandler::setAccelRange(uint8_t range) {
  accel_range_ = range & 0x03;
  uint8_t value = accel_range_ << 3;
  writeRegister(MPU6050_ACCEL_CONFIG, value);
  
  switch (accel_range_) {
    case 0: accel_scale_ = 16384.0f; break;  // 2g
    case 1: accel_scale_ = 8192.0f; break;   // 4g
    case 2: accel_scale_ = 4096.0f; break;   // 8g
    case 3: accel_scale_ = 2048.0f; break;   // 16g
  }
}

void IMUHandler::setGyroRange(uint8_t range) {
  gyro_range_ = range & 0x03;
  uint8_t value = gyro_range_ << 3;
  writeRegister(MPU6050_GYRO_CONFIG, value);
  
  switch (gyro_range_) {
    case 0: gyro_scale_ = 131.0f; break;    // 250 dps
    case 1: gyro_scale_ = 65.5f; break;     // 500 dps
    case 2: gyro_scale_ = 32.8f; break;     // 1000 dps
    case 3: gyro_scale_ = 16.4f; break;     // 2000 dps
  }
}

void IMUHandler::computeOrientation() {
  uint32_t now = millis();
  float dt = (now - last_orientation_ms_) / 1000.0f;
  if (dt <= 0 || dt > 1.0f) dt = 0.01f;
  last_orientation_ms_ = now;
  
  // Accelerometer-based angles
  float accel_roll = atan2(data_.ay_g, data_.az_g) * 180.0f / PI;
  float accel_pitch = atan2(-data_.ax_g, sqrt(data_.ay_g * data_.ay_g + data_.az_g * data_.az_g)) * 180.0f / PI;
  
  // Gyro integration
  roll_ += data_.gx_dps * dt;
  pitch_ += data_.gy_dps * dt;
  yaw_ += data_.gz_dps * dt;
  
  // Complementary filter (0.98 gyro, 0.02 accel)
  const float alpha = 0.98f;
  roll_ = alpha * roll_ + (1.0f - alpha) * accel_roll;
  pitch_ = alpha * pitch_ + (1.0f - alpha) * accel_pitch;
  
  data_.roll = roll_;
  data_.pitch = pitch_;
  data_.yaw = yaw_;
}

void IMUHandler::resetOrientation() {
  roll_ = data_.roll;
  pitch_ = data_.pitch;
  yaw_ = 0;
}

bool IMUHandler::selfTest() {
  if (!connected_) return false;
  
  // Enable self-test
  writeRegister(MPU6050_ACCEL_CONFIG, (accel_range_ << 3) | 0xE0);
  writeRegister(MPU6050_GYRO_CONFIG, (gyro_range_ << 3) | 0xE0);
  delay(100);
  
  int16_t ax, ay, az, gx, gy, gz, temp;
  readRaw(&ax, &ay, &az, &gx, &gy, &gz, &temp);
  
  // Disable self-test
  writeRegister(MPU6050_ACCEL_CONFIG, accel_range_ << 3);
  writeRegister(MPU6050_GYRO_CONFIG, gyro_range_ << 3);
  
  // Check if values changed significantly (simplified check)
  return (abs(ax) > 100 || abs(ay) > 100 || abs(az) > 100);
}

uint32_t IMUHandler::getSampleCount() const {
  return sample_count_;
}

uint32_t IMUHandler::getErrorCount() const {
  return error_count_;
}

float IMUHandler::getTemperatureC() const {
  return data_.temp_c;
}

bool IMUHandler::writeRegister(uint8_t reg, uint8_t value) {
  Wire.beginTransmission(MPU6050_ADDR);
  Wire.write(reg);
  Wire.write(value);
  return Wire.endTransmission() == 0;
}

bool IMUHandler::readRegister(uint8_t reg, uint8_t* value) {
  Wire.beginTransmission(MPU6050_ADDR);
  Wire.write(reg);
  if (Wire.endTransmission(false) != 0) return false;
  
  Wire.requestFrom(MPU6050_ADDR, (uint8_t)1);
  if (Wire.available()) {
    *value = Wire.read();
    return true;
  }
  return false;
}

bool IMUHandler::readRegisters(uint8_t reg, uint8_t* buffer, uint8_t len) {
  Wire.beginTransmission(MPU6050_ADDR);
  Wire.write(reg);
  if (Wire.endTransmission(false) != 0) return false;
  
  Wire.requestFrom(MPU6050_ADDR, len);
  uint8_t i = 0;
  while (Wire.available() && i < len) {
    buffer[i++] = Wire.read();
  }
  return i == len;
}

float IMUHandler::accelToG(int16_t raw) const {
  return raw / accel_scale_;
}

float IMUHandler::gyroToDPS(int16_t raw) const {
  return raw / gyro_scale_;
}

float IMUHandler::tempToC(int16_t raw) const {
  return (raw / 340.0f) + 36.53f;
}