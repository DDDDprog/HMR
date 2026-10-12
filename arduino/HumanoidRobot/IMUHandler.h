#ifndef IMU_HANDLER_H
#define IMU_HANDLER_H

#include "Config.h"
#include <Wire.h>

struct IMUData {
  int16_t ax, ay, az;
  int16_t gx, gy, gz;
  int16_t temp;
  float ax_g, ay_g, az_g;
  float gx_dps, gy_dps, gz_dps;
  float temp_c;
  float roll, pitch, yaw;  // Degrees
  uint32_t timestamp_ms;
};

struct IMUOffsets {
  int16_t ax_off, ay_off, az_off;
  int16_t gx_off, gy_off, gz_off;
  bool valid;
};

class IMUHandler {
public:
  IMUHandler();
  
  bool begin();
  bool update();
  bool isConnected() const;
  
  // Data access
  const IMUData* getData() const;
  bool readRaw(int16_t* ax, int16_t* ay, int16_t* az, int16_t* gx, int16_t* gy, int16_t* gz, int16_t* temp);
  
  // Calibration
  bool calibrate(uint16_t samples = 1000);
  bool setOffsets(const IMUOffsets* offsets);
  const IMUOffsets* getOffsets() const;
  bool saveOffsets();
  bool loadOffsets();
  
  // Configuration
  void setSampleRate(uint8_t divider);  // 1kHz / (1 + divider)
  void setDLPF(uint8_t config);  // Digital Low Pass Filter
  void setAccelRange(uint8_t range);  // 0=2g, 1=4g, 2=8g, 3=16g
  void setGyroRange(uint8_t range);   // 0=250, 1=500, 2=1000, 3=2000 dps
  
  // Orientation
  void computeOrientation();
  void resetOrientation();
  
  // Self-test
  bool selfTest();
  
  // Status
  uint32_t getSampleCount() const;
  uint32_t getErrorCount() const;
  float getTemperatureC() const;
  
private:
  IMUData data_;
  IMUOffsets offsets_;
  bool connected_;
  uint32_t sample_count_;
  uint32_t error_count_;
  uint32_t last_update_ms_;
  
  // Orientation filter (complementary)
  float roll_, pitch_, yaw_;
  uint32_t last_orientation_ms_;
  
  // Register access
  bool writeRegister(uint8_t reg, uint8_t value);
  bool readRegister(uint8_t reg, uint8_t* value);
  bool readRegisters(uint8_t reg, uint8_t* buffer, uint8_t len);
  
  // Conversion
  float accelToG(int16_t raw) const;
  float gyroToDPS(int16_t raw) const;
  float tempToC(int16_t raw) const;
  
  // Configuration state
  uint8_t accel_range_;  // 0-3
  uint8_t gyro_range_;   // 0-3
  float accel_scale_;
  float gyro_scale_;
};

#endif // IMU_HANDLER_H