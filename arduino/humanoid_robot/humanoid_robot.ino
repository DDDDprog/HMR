/*
 * Humanoid Robot Firmware for Arduino Mega 2560
 * 
 * Hardware:
 * - 14 hobby servos on pins D2-D13, D22-D23
 * - GY-521 MPU-6050 IMU on I2C (SDA=D20, SCL=D21)
 * - USB serial communication at 115200 baud
 * 
 * Safety:
 * - Servos start disabled
 * - No automatic movement on startup
 * - Per-servo angle limits enforced
 * - External power required for servos
 */

#include <Wire.h>
#include <Servo.h>

// Serial configuration
#define BAUD_RATE 115200
#define SERIAL_TIMEOUT 100

// Servo configuration
#define NUM_SERVOS 14
#define SERVO_UPDATE_DELAY_MS 20

// MPU-6050 I2C address
#define MPU_ADDR 0x68

// Servo pin assignments
const uint8_t SERVO_PINS[NUM_SERVOS] = {
  2,   // 0: left hip
  3,   // 1: left knee
  4,   // 2: left ankle
  5,   // 3: right hip
  6,   // 4: right knee
  7,   // 5: right ankle
  8,   // 6: left shoulder
  9,   // 7: left elbow
  10,  // 8: left claw/wrist
  11,  // 9: right shoulder
  12,  // 10: right elbow
  13,  // 11: right claw/wrist
  22,  // 12: head pan
  23   // 13: head tilt
};

// Servo names for error messages
const char* SERVO_NAMES[NUM_SERVOS] = {
  "left hip",
  "left knee",
  "left ankle",
  "right hip",
  "right knee",
  "right ankle",
  "left shoulder",
  "left elbow",
  "left claw/wrist",
  "right shoulder",
  "right elbow",
  "right claw/wrist",
  "head pan",
  "head tilt"
};

// Conservative initial angle limits (degrees)
// These must be calibrated to actual mechanical limits
struct ServoLimits {
  int min_angle;
  int max_angle;
  int home_angle;
};

ServoLimits SERVO_LIMITS[NUM_SERVOS] = {
  { -90, 90, 0 },    // left hip
  { -90, 90, 0 },    // left knee
  { -45, 45, 0 },    // left ankle
  { -90, 90, 0 },    // right hip
  { -90, 90, 0 },    // right knee
  { -45, 45, 0 },    // right ankle
  { -90, 90, 0 },    // left shoulder
  { -90, 90, 0 },    // left elbow
  { -90, 90, 0 },    // left claw/wrist
  { -90, 90, 0 },    // right shoulder
  { -90, 90, 0 },    // right elbow
  { -90, 90, 0 },    // right claw/wrist
  { -90, 90, 0 },    // head pan
  { -45, 45, 0 }     // head tilt
};

// Global state
Servo servos[NUM_SERVOS];
int current_angles[NUM_SERVOS] = {0};
bool servos_enabled = false;
bool mpu_initialized = false;

// MPU-6050 data
int16_t mpu_accel_x, mpu_accel_y, mpu_accel_z;
int16_t mpu_gyro_x, mpu_gyro_y, mpu_gyro_z;
int16_t mpu_temp;

void setup() {
  // Initialize serial
  Serial.begin(BAUD_RATE);
  Serial.setTimeout(SERIAL_TIMEOUT);
  
  // Initialize I2C for MPU-6050
  Wire.begin();
  
  // Initialize servo objects (but don't attach yet)
  for (int i = 0; i < NUM_SERVOS; i++) {
    current_angles[i] = SERVO_LIMITS[i].home_angle;
  }
  
  // Attempt to initialize MPU-6050
  init_mpu();
  
  // Send ready message
  Serial.println("READY");
}

void loop() {
  process_serial_commands();
  delay(SERVO_UPDATE_DELAY_MS);
}

// Initialize MPU-6050
void init_mpu() {
  Wire.beginTransmission(MPU_ADDR);
  Wire.write(0x6B);  // PWR_MGMT_1 register
  Wire.write(0);     // Wake up MPU-6050
  Wire.endTransmission(true);
  
  // Simple check if MPU responds
  Wire.beginTransmission(MPU_ADDR);
  Wire.write(0x75);  // WHO_AM_I register
  Wire.endTransmission(false);
  Wire.requestFrom(MPU_ADDR, 1, true);
  
  if (Wire.available() && Wire.read() == 0x68) {
    mpu_initialized = true;
  }
}

// Read MPU-6050 data
void read_mpu() {
  if (!mpu_initialized) {
    return;
  }
  
  Wire.beginTransmission(MPU_ADDR);
  Wire.write(0x3B);  // ACCEL_XOUT_H register
  Wire.endTransmission(false);
  Wire.requestFrom(MPU_ADDR, 14, true);
  
  if (Wire.available() >= 14) {
    mpu_accel_x = Wire.read() << 8 | Wire.read();
    mpu_accel_y = Wire.read() << 8 | Wire.read();
    mpu_accel_z = Wire.read() << 8 | Wire.read();
    mpu_temp = Wire.read() << 8 | Wire.read();
    mpu_gyro_x = Wire.read() << 8 | Wire.read();
    mpu_gyro_y = Wire.read() << 8 | Wire.read();
    mpu_gyro_z = Wire.read() << 8 | Wire.read();
  }
}

// Process serial commands
void process_serial_commands() {
  if (Serial.available() == 0) {
    return;
  }
  
  String command = Serial.readStringUntil('\n');
  command.trim();
  
  if (command.length() == 0) {
    return;
  }
  
  // Parse command: COMMAND [ARG1] [ARG2] ...
  int space_idx = command.indexOf(' ');
  String cmd;
  String args;
  
  if (space_idx == -1) {
    cmd = command;
  } else {
    cmd = command.substring(0, space_idx);
    args = command.substring(space_idx + 1);
  }
  
  // Process commands
  if (cmd == "ENABLE") {
    enable_servos();
    Serial.println("OK");
  } else if (cmd == "DISABLE") {
    disable_servos();
    Serial.println("OK");
  } else if (cmd == "SET") {
    handle_set_command(args);
  } else if (cmd == "GET") {
    handle_get_command(args);
  } else if (cmd == "HOME") {
    home_servos();
    Serial.println("OK");
  } else if (cmd == "MPU") {
    read_mpu();
    Serial.print("MPU ");
    Serial.print(mpu_accel_x);
    Serial.print(" ");
    Serial.print(mpu_accel_y);
    Serial.print(" ");
    Serial.print(mpu_accel_z);
    Serial.print(" ");
    Serial.print(mpu_gyro_x);
    Serial.print(" ");
    Serial.print(mpu_gyro_y);
    Serial.print(" ");
    Serial.println(mpu_gyro_z);
  } else if (cmd == "STATUS") {
    send_status();
  } else if (cmd == "PING") {
    Serial.println("PONG");
  } else {
    Serial.print("ERROR unknown command: ");
    Serial.println(cmd);
  }
}

// Enable servos (attach to pins)
void enable_servos() {
  if (servos_enabled) {
    return;
  }
  
  for (int i = 0; i < NUM_SERVOS; i++) {
    servos[i].attach(SERVO_PINS[i]);
    servos[i].write(angle_to_pwm(current_angles[i]));
  }
  
  servos_enabled = true;
}

// Disable servos (detach from pins)
void disable_servos() {
  if (!servos_enabled) {
    return;
  }
  
  for (int i = 0; i < NUM_SERVOS; i++) {
    servos[i].detach();
  }
  
  servos_enabled = false;
}

// Handle SET command: SET <index> <angle>
void handle_set_command(String args) {
  int space_idx = args.indexOf(' ');
  if (space_idx == -1) {
    Serial.println("ERROR invalid SET syntax");
    return;
  }
  
  String index_str = args.substring(0, space_idx);
  String angle_str = args.substring(space_idx + 1);
  
  int index = index_str.toInt();
  int angle = angle_str.toInt();
  
  if (index < 0 || index >= NUM_SERVOS) {
    Serial.print("ERROR invalid servo index: ");
    Serial.println(index);
    return;
  }
  
  if (!validate_angle(index, angle)) {
    Serial.print("ERROR angle out of bounds for ");
    Serial.print(SERVO_NAMES[index]);
    Serial.print(": ");
    Serial.print(angle);
    Serial.print(" (min=");
    Serial.print(SERVO_LIMITS[index].min_angle);
    Serial.print(", max=");
    Serial.print(SERVO_LIMITS[index].max_angle);
    Serial.println(")");
    return;
  }
  
  current_angles[index] = angle;
  
  if (servos_enabled) {
    servos[index].write(angle_to_pwm(angle));
  }
  
  Serial.println("OK");
}

// Handle GET command: GET <index>
void handle_get_command(String args) {
  int index = args.toInt();
  
  if (index < 0 || index >= NUM_SERVOS) {
    Serial.print("ERROR invalid servo index: ");
    Serial.println(index);
    return;
  }
  
  Serial.print("GET ");
  Serial.print(index);
  Serial.print(" ");
  Serial.println(current_angles[index]);
}

// Validate angle against limits
bool validate_angle(int index, int angle) {
  if (index < 0 || index >= NUM_SERVOS) {
    return false;
  }
  
  return angle >= SERVO_LIMITS[index].min_angle && 
         angle <= SERVO_LIMITS[index].max_angle;
}

// Convert angle (-90 to 90) to PWM (0 to 180)
int angle_to_pwm(int angle) {
  return angle + 90;
}

// Move all servos to home position
void home_servos() {
  for (int i = 0; i < NUM_SERVOS; i++) {
    current_angles[i] = SERVO_LIMITS[i].home_angle;
    if (servos_enabled) {
      servos[i].write(angle_to_pwm(current_angles[i]));
    }
  }
}

// Send status information
void send_status() {
  Serial.print("STATUS enabled=");
  Serial.print(servos_enabled ? "true" : "false");
  Serial.print(" mpu=");
  Serial.print(mpu_initialized ? "true" : "false");
  Serial.print(" servos=");
  
  for (int i = 0; i < NUM_SERVOS; i++) {
    Serial.print(current_angles[i]);
    if (i < NUM_SERVOS - 1) {
      Serial.print(",");
    }
  }
  
  Serial.println();
}
