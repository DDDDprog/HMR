use anyhow::{Context, Result};
use serialport::{SerialPort, SerialPortType};
use std::io::{Read, Write};
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::config::SerialConfig;

// Protocol constants (must match Arduino firmware)
const PROTOCOL_START: u8 = 0xAA;
const PROTOCOL_END: u8 = 0x55;
const MAX_PAYLOAD: usize = 64;

// Command IDs
const CMD_PING: u8 = 0x01;
const CMD_GET_VERSION: u8 = 0x02;
const CMD_SET_SERVO_ANGLE: u8 = 0x10;
const CMD_SET_SERVO_PULSE: u8 = 0x11;
const CMD_GET_SERVO_STATE: u8 = 0x12;
const CMD_SET_ALL_SERVOS: u8 = 0x13;
const CMD_SET_SERVO_LIMITS: u8 = 0x20;
const CMD_GET_SERVO_LIMITS: u8 = 0x21;
const CMD_SAVE_CONFIG: u8 = 0x22;
const CMD_LOAD_CONFIG: u8 = 0x23;
const CMD_RESET_SERVOS: u8 = 0x30;
const CMD_EMERGENCY_STOP: u8 = 0x31;
const CMD_GET_IMU_DATA: u8 = 0x40;
const CMD_CALIBRATE_IMU: u8 = 0x41;
const CMD_SET_IMU_OFFSETS: u8 = 0x42;
const CMD_GET_IMU_OFFSETS: u8 = 0x43;
const CMD_START_STREAM: u8 = 0x50;
const CMD_STOP_STREAM: u8 = 0x51;
const CMD_SET_STREAM_RATE: u8 = 0x52;
const CMD_RUN_SEQUENCE: u8 = 0x60;
const CMD_STOP_SEQUENCE: u8 = 0x61;
const CMD_GET_DIAGNOSTICS: u8 = 0x70;

// Response IDs
const RSP_ACK: u8 = 0x80;
const RSP_NACK: u8 = 0x81;
const RSP_VERSION: u8 = 0x82;
const RSP_SERVO_STATE: u8 = 0x90;
const RSP_ALL_SERVO_STATE: u8 = 0x91;
const RSP_SERVO_LIMITS: u8 = 0x92;
const RSP_IMU_DATA: u8 = 0xA0;
const RSP_IMU_OFFSETS: u8 = 0xA1;
const RSP_DIAGNOSTICS: u8 = 0xB0;
const RSP_ERROR: u8 = 0xFF;

// Error codes
const ERR_NONE: u8 = 0x00;
const ERR_INVALID_CMD: u8 = 0x01;
const ERR_INVALID_PARAM: u8 = 0x02;
const ERR_CHECKSUM_FAIL: u8 = 0x03;
const ERR_SERVO_INDEX: u8 = 0x04;
const ERR_ANGLE_OUT_OF_RANGE: u8 = 0x05;
const ERR_PULSE_OUT_OF_RANGE: u8 = 0x06;
const ERR_IMU_NOT_FOUND: u8 = 0x07;
const ERR_IMU_INIT_FAIL: u8 = 0x08;
const ERR_EEPROM_WRITE_FAIL: u8 = 0x09;
const ERR_SAFETY_VIOLATION: u8 = 0x0A;
const ERR_COMM_TIMEOUT: u8 = 0x0B;
const ERR_SEQUENCE_RUNNING: u8 = 0x0C;

#[derive(Debug, Clone)]
pub struct ServoState {
    pub index: u8,
    pub angle: f32,    // degrees
    pub pulse_us: u16, // microseconds
    pub attached: bool,
}

#[derive(Debug, Clone)]
pub struct ServoLimits {
    pub index: u8,
    pub min_angle: f32,
    pub max_angle: f32,
    pub min_pulse: u16,
    pub max_pulse: u16,
    pub center_pulse: u16,
}

#[derive(Debug, Clone)]
pub struct IMUData {
    pub ax: i16,
    pub ay: i16,
    pub az: i16,
    pub gx: i16,
    pub gy: i16,
    pub gz: i16,
    pub temp: i16,
    pub ax_g: f32,
    pub ay_g: f32,
    pub az_g: f32,
    pub gx_dps: f32,
    pub gy_dps: f32,
    pub gz_dps: f32,
    pub temp_c: f32,
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub timestamp_ms: u32,
}

#[derive(Debug, Clone)]
pub struct Diagnostics {
    pub uptime_ms: u32,
    pub free_ram: u16,
    pub servo_currents: [u16; 14],
    pub total_current_ma: u16,
    pub voltage_mv: u16,
    pub temperature_c: i16,
    pub packets_rx: u32,
    pub packets_tx: u32,
    pub crc_errors: u32,
    pub timeout_errors: u32,
    pub emergency_stop: bool,
    pub imu_connected: bool,
    pub firmware_version: [u8; 3],
}

#[derive(Debug, Clone)]
pub enum Response {
    Ack(u8),
    Nack(u8, u8),
    Version(u8, u8, u8),
    ServoState(ServoState),
    ServoLimits(ServoLimits),
    IMUData(IMUData),
    Diagnostics(Diagnostics),
    Error(u8),
}

pub struct SerialManager {
    config: SerialConfig,
    port: Option<Box<dyn SerialPort>>,
    rx_buffer: Vec<u8>,
    tx_buffer: Vec<u8>,
    connected: bool,
    last_response: Option<Response>,
}

impl SerialManager {
    pub fn new(config: SerialConfig) -> Self {
        Self {
            config,
            port: None,
            rx_buffer: Vec::with_capacity(256),
            tx_buffer: Vec::with_capacity(256),
            connected: false,
            last_response: None,
        }
    }

    pub fn list_ports() -> Result<Vec<SerialPortInfo>> {
        let ports = serialport::available_ports()?;
        Ok(ports.into_iter().map(Into::into).collect())
    }

    pub fn set_port(&mut self, port: String) {
        self.config.port = port;
    }

    pub fn set_baud_rate(&mut self, baud_rate: u32) {
        self.config.baud_rate = baud_rate;
    }

    pub fn connect(&mut self) -> Result<()> {
        if self.connected {
            return Ok(());
        }

        let port = serialport::new(&self.config.port, self.config.baud_rate)
            .timeout(Duration::from_millis(self.config.timeout_ms))
            .open()
            .context("Failed to open serial port")?;

        self.port = Some(port);
        self.connected = true;
        self.rx_buffer.clear();
        self.tx_buffer.clear();

        // Wait for Arduino to reset
        std::thread::sleep(Duration::from_millis(500));

        // Clear any startup garbage
        if let Err(error) = self.clear_input_buffer() {
            self.disconnect();
            return Err(error);
        }

        info!("Connected to {}", self.config.port);
        Ok(())
    }

    pub fn disconnect(&mut self) {
        if let Some(mut port) = self.port.take() {
            let _ = port.flush();
        }
        self.connected = false;
        info!("Disconnected");
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn send_ping(&mut self) -> Result<bool> {
        self.send_command(CMD_PING, &[])?;

        let deadline = std::time::Instant::now() + Duration::from_millis(500);
        while std::time::Instant::now() < deadline {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            match self.read_response(remaining)? {
                Some(Response::Ack(cmd)) if cmd == CMD_PING => return Ok(true),
                Some(_) => continue,
                None => return Ok(false),
            }
        }

        Ok(false)
    }

    pub fn get_version(&mut self) -> Result<Option<(u8, u8, u8)>> {
        self.send_command(CMD_GET_VERSION, &[])?;
        match self.wait_for_response(Duration::from_millis(500), |response| {
            matches!(response, Response::Version(..))
        })? {
            Some(Response::Version(major, minor, patch)) => Ok(Some((major, minor, patch))),
            _ => Ok(None),
        }
    }

    pub fn set_servo_angle(&mut self, index: u8, angle: f32) -> Result<bool> {
        let angle_x10 = (angle * 10.0) as u16;
        let payload = [index, (angle_x10 >> 8) as u8, (angle_x10 & 0xFF) as u8];
        self.send_command(CMD_SET_SERVO_ANGLE, &payload)?;
        self.wait_for_ack(
            CMD_SET_SERVO_ANGLE,
            Duration::from_millis(200),
            "Servo command",
        )
    }

    pub fn set_servo_pulse(&mut self, index: u8, pulse_us: u16) -> Result<bool> {
        let payload = [index, (pulse_us >> 8) as u8, (pulse_us & 0xFF) as u8];
        self.send_command(CMD_SET_SERVO_PULSE, &payload)?;
        self.wait_for_ack(
            CMD_SET_SERVO_PULSE,
            Duration::from_millis(200),
            "Servo command",
        )
    }

    pub fn get_servo_state(&mut self, index: u8) -> Result<Option<ServoState>> {
        self.send_command(CMD_GET_SERVO_STATE, &[index])?;
        match self.wait_for_response(Duration::from_millis(200), |response| {
            matches!(response, Response::ServoState(_))
                || matches!(response, Response::Nack(command, _) if *command == CMD_GET_SERVO_STATE)
        })? {
            Some(Response::ServoState(state)) => Ok(Some(state)),
            Some(Response::Nack(_, err)) => Err(anyhow::anyhow!(
                "Servo state request rejected: {}",
                error_to_str(err)
            )),
            _ => Ok(None),
        }
    }

    pub fn set_all_servos(&mut self, angles: &[f32; 14]) -> Result<bool> {
        let mut payload = Vec::with_capacity(28);
        for angle in angles {
            let angle_x10 = (angle * 10.0) as u16;
            payload.push((angle_x10 >> 8) as u8);
            payload.push((angle_x10 & 0xFF) as u8);
        }
        self.send_command(CMD_SET_ALL_SERVOS, &payload)?;
        self.wait_for_ack(
            CMD_SET_ALL_SERVOS,
            Duration::from_millis(500),
            "Servo command",
        )
    }

    pub fn set_servo_limits(&mut self, index: u8, limits: &ServoLimits) -> Result<bool> {
        let payload = [
            index,
            ((limits.min_angle * 10.0) as u16 >> 8) as u8,
            ((limits.min_angle * 10.0) as u16 & 0xFF) as u8,
            ((limits.max_angle * 10.0) as u16 >> 8) as u8,
            ((limits.max_angle * 10.0) as u16 & 0xFF) as u8,
            (limits.min_pulse >> 8) as u8,
            (limits.min_pulse & 0xFF) as u8,
            (limits.max_pulse >> 8) as u8,
            (limits.max_pulse & 0xFF) as u8,
            (limits.center_pulse >> 8) as u8,
            (limits.center_pulse & 0xFF) as u8,
        ];
        self.send_command(CMD_SET_SERVO_LIMITS, &payload)?;
        self.wait_for_ack(
            CMD_SET_SERVO_LIMITS,
            Duration::from_millis(200),
            "Limit update",
        )
    }

    pub fn get_servo_limits(&mut self, index: u8) -> Result<Option<ServoLimits>> {
        self.send_command(CMD_GET_SERVO_LIMITS, &[index])?;
        match self.wait_for_response(Duration::from_millis(200), |response| {
            matches!(response, Response::ServoLimits(_))
                || matches!(response, Response::Nack(command, _) if *command == CMD_GET_SERVO_LIMITS)
        })? {
            Some(Response::ServoLimits(limits)) => Ok(Some(limits)),
            Some(Response::Nack(_, err)) => Err(anyhow::anyhow!(
                "Servo limit request rejected: {}",
                error_to_str(err)
            )),
            _ => Ok(None),
        }
    }

    pub fn save_config(&mut self) -> Result<bool> {
        self.send_command(CMD_SAVE_CONFIG, &[])?;
        self.wait_for_ack(CMD_SAVE_CONFIG, Duration::from_millis(1000), "Config save")
    }

    pub fn load_config(&mut self) -> Result<bool> {
        self.send_command(CMD_LOAD_CONFIG, &[])?;
        self.wait_for_ack(CMD_LOAD_CONFIG, Duration::from_millis(500), "Config load")
    }

    pub fn reset_servos(&mut self) -> Result<bool> {
        self.send_command(CMD_RESET_SERVOS, &[])?;
        self.wait_for_ack(CMD_RESET_SERVOS, Duration::from_millis(500), "Servo reset")
    }

    pub fn emergency_stop(&mut self) -> Result<bool> {
        self.send_command(CMD_EMERGENCY_STOP, &[])?;
        self.wait_for_ack(
            CMD_EMERGENCY_STOP,
            Duration::from_millis(500),
            "Emergency stop",
        )
    }

    pub fn get_imu_data(&mut self) -> Result<Option<IMUData>> {
        self.send_command(CMD_GET_IMU_DATA, &[])?;
        match self.wait_for_response(Duration::from_millis(200), |response| {
            matches!(response, Response::IMUData(_))
                || matches!(response, Response::Nack(command, _) if *command == CMD_GET_IMU_DATA)
        })? {
            Some(Response::IMUData(data)) => Ok(Some(data)),
            Some(Response::Nack(_, ERR_IMU_NOT_FOUND)) => Ok(None),
            Some(Response::Nack(_, error)) => Err(anyhow::anyhow!(
                "IMU request rejected: {}",
                error_to_str(error)
            )),
            _ => Ok(None),
        }
    }

    pub fn calibrate_imu(&mut self) -> Result<bool> {
        self.send_command(CMD_CALIBRATE_IMU, &[])?;
        self.wait_for_ack(
            CMD_CALIBRATE_IMU,
            Duration::from_millis(5000),
            "IMU calibration",
        )
    }

    pub fn start_stream(&mut self, stream_type: u8, rate_ms: u16) -> Result<bool> {
        let payload = [stream_type, (rate_ms >> 8) as u8, (rate_ms & 0xFF) as u8];
        self.send_command(CMD_START_STREAM, &payload)?;
        self.wait_for_ack(CMD_START_STREAM, Duration::from_millis(200), "Stream start")
    }

    pub fn stop_stream(&mut self) -> Result<bool> {
        self.send_command(CMD_STOP_STREAM, &[])?;
        self.wait_for_ack(CMD_STOP_STREAM, Duration::from_millis(200), "Stream stop")
    }

    pub fn get_diagnostics(&mut self) -> Result<Option<Diagnostics>> {
        self.send_command(CMD_GET_DIAGNOSTICS, &[])?;
        match self.wait_for_response(Duration::from_millis(500), |response| {
            matches!(response, Response::Diagnostics(_))
        })? {
            Some(Response::Diagnostics(diag)) => Ok(Some(diag)),
            _ => Ok(None),
        }
    }

    pub fn poll_stream(&mut self) -> Result<Vec<Response>> {
        let mut responses = Vec::new();
        let mut timeout = Duration::from_millis(10);

        while let Some(response) = self.read_response(timeout)? {
            responses.push(response);
            timeout = Duration::from_millis(1); // Subsequent reads with minimal timeout
        }

        Ok(responses)
    }

    fn send_command(&mut self, cmd: u8, payload: &[u8]) -> Result<()> {
        if !self.connected {
            return Err(anyhow::anyhow!("Not connected"));
        }

        if payload.len() > MAX_PAYLOAD {
            return Err(anyhow::anyhow!("Payload too large"));
        }

        self.tx_buffer.clear();
        self.tx_buffer.push(PROTOCOL_START);
        self.tx_buffer.push(cmd);
        self.tx_buffer.push(payload.len() as u8);
        self.tx_buffer.extend_from_slice(payload);

        let checksum = self.calculate_checksum(&self.tx_buffer[1..]);
        self.tx_buffer.push(checksum);
        self.tx_buffer.push(PROTOCOL_END);

        if let Some(port) = &mut self.port {
            port.write_all(&self.tx_buffer)
                .context("Failed to write to serial port")?;
            port.flush().context("Failed to flush serial port")?;
        }

        debug!("TX: {:02X?}", self.tx_buffer);
        Ok(())
    }

    fn wait_for_ack(&mut self, command: u8, timeout: Duration, operation: &str) -> Result<bool> {
        match self.wait_for_response(timeout, |response| match response {
            Response::Ack(acknowledged) => *acknowledged == command,
            Response::Nack(rejected, _) => *rejected == command,
            _ => false,
        })? {
            Some(Response::Ack(_)) => Ok(true),
            Some(Response::Nack(_, error)) => Err(anyhow::anyhow!(
                "{operation} rejected: {}",
                error_to_str(error)
            )),
            _ => Ok(false),
        }
    }

    fn wait_for_response<F>(
        &mut self,
        timeout: Duration,
        mut matches_response: F,
    ) -> Result<Option<Response>>
    where
        F: FnMut(&Response) -> bool,
    {
        let deadline = std::time::Instant::now() + timeout;
        while std::time::Instant::now() < deadline {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            match self.read_response(remaining)? {
                Some(response) if matches_response(&response) => return Ok(Some(response)),
                Some(_) => continue,
                None => return Ok(None),
            }
        }
        Ok(None)
    }

    fn read_response(&mut self, timeout: Duration) -> Result<Option<Response>> {
        if !self.connected {
            return Ok(None);
        }

        let deadline = std::time::Instant::now() + timeout;

        while std::time::Instant::now() < deadline {
            if let Some(port) = &mut self.port {
                let read_timeout = deadline
                    .saturating_duration_since(std::time::Instant::now())
                    .min(Duration::from_millis(20))
                    .max(Duration::from_millis(1));
                port.set_timeout(read_timeout)
                    .context("Failed to set serial read timeout")?;
                let mut byte = [0u8];
                match port.read(&mut byte) {
                    Ok(n) if n == 1 => {
                        self.rx_buffer.push(byte[0]);
                        if let Some(response) = self.try_parse_packet()? {
                            debug!("RX: {:?}", response);
                            return Ok(Some(response));
                        }
                    }
                    Ok(_) => {
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    Err(e) => {
                        error!("Serial read error: {}", e);
                        self.connected = false;
                        return Err(anyhow::anyhow!("Serial read error: {}", e));
                    }
                }
            } else {
                return Ok(None);
            }
        }

        Ok(None)
    }

    fn try_parse_packet(&mut self) -> Result<Option<Response>> {
        // Find start byte
        let start_pos = self.rx_buffer.iter().position(|&b| b == PROTOCOL_START);
        if start_pos.is_none() {
            // No start byte, clear buffer if too large
            if self.rx_buffer.len() > 256 {
                self.rx_buffer.clear();
            }
            return Ok(None);
        }

        let start = start_pos.unwrap();
        if start > 0 {
            self.rx_buffer.drain(0..start);
        }

        // Need at least 5 bytes (start, cmd, len, checksum, end)
        if self.rx_buffer.len() < 5 {
            return Ok(None);
        }

        let payload_len = self.rx_buffer[2] as usize;
        if payload_len > MAX_PAYLOAD {
            self.rx_buffer.clear();
            return Ok(None);
        }

        let expected_len = 5 + payload_len;
        if self.rx_buffer.len() < expected_len {
            return Ok(None);
        }

        // Check end byte
        if self.rx_buffer[expected_len - 1] != PROTOCOL_END {
            self.rx_buffer.drain(0..1); // Remove start and retry
            return Ok(None);
        }

        // Extract packet
        let cmd = self.rx_buffer[1];
        let payload = self.rx_buffer[3..3 + payload_len].to_vec();
        let checksum = self.rx_buffer[3 + payload_len];

        // Verify checksum
        let mut calc_data = vec![cmd, payload_len as u8];
        calc_data.extend_from_slice(&payload);
        let calc_checksum = self.calculate_checksum(&calc_data);

        if checksum != calc_checksum {
            warn!("Checksum mismatch");
            self.rx_buffer.drain(0..1);
            return Ok(None);
        }

        // Parse response
        let response = self.parse_response(cmd, &payload);

        // Remove processed packet
        self.rx_buffer.drain(0..expected_len);

        Ok(Some(response))
    }

    fn calculate_checksum(&self, data: &[u8]) -> u8 {
        data.iter().fold(0u8, |acc, &b| acc ^ b)
    }

    fn parse_response(&self, cmd: u8, payload: &[u8]) -> Response {
        match cmd {
            RSP_ACK => Response::Ack(payload.get(0).copied().unwrap_or(0)),
            RSP_NACK => Response::Nack(
                payload.get(0).copied().unwrap_or(0),
                payload.get(1).copied().unwrap_or(0),
            ),
            RSP_VERSION => Response::Version(
                payload.get(0).copied().unwrap_or(0),
                payload.get(1).copied().unwrap_or(0),
                payload.get(2).copied().unwrap_or(0),
            ),
            RSP_SERVO_STATE => {
                if payload.len() >= 6 {
                    Response::ServoState(ServoState {
                        index: payload[0],
                        angle: ((payload[1] as u16) << 8 | payload[2] as u16) as f32 / 10.0,
                        pulse_us: (payload[3] as u16) << 8 | payload[4] as u16,
                        attached: payload[5] != 0,
                    })
                } else {
                    Response::Error(ERR_INVALID_PARAM)
                }
            }
            RSP_SERVO_LIMITS => {
                if payload.len() >= 11 {
                    Response::ServoLimits(ServoLimits {
                        index: payload[0],
                        min_angle: ((payload[1] as u16) << 8 | payload[2] as u16) as f32 / 10.0,
                        max_angle: ((payload[3] as u16) << 8 | payload[4] as u16) as f32 / 10.0,
                        min_pulse: (payload[5] as u16) << 8 | payload[6] as u16,
                        max_pulse: (payload[7] as u16) << 8 | payload[8] as u16,
                        center_pulse: (payload[9] as u16) << 8 | payload[10] as u16,
                    })
                } else {
                    Response::Error(ERR_INVALID_PARAM)
                }
            }
            RSP_IMU_DATA => {
                if payload.len() >= 30 {
                    let ax = (payload[0] as i16) << 8 | payload[1] as i16;
                    let ay = (payload[2] as i16) << 8 | payload[3] as i16;
                    let az = (payload[4] as i16) << 8 | payload[5] as i16;
                    let gx = (payload[6] as i16) << 8 | payload[7] as i16;
                    let gy = (payload[8] as i16) << 8 | payload[9] as i16;
                    let gz = (payload[10] as i16) << 8 | payload[11] as i16;
                    let temp = (payload[12] as i16) << 8 | payload[13] as i16;

                    // Parse floats (little endian)
                    let ax_g =
                        f32::from_le_bytes([payload[14], payload[15], payload[16], payload[17]]);
                    let ay_g =
                        f32::from_le_bytes([payload[18], payload[19], payload[20], payload[21]]);
                    let az_g =
                        f32::from_le_bytes([payload[22], payload[23], payload[24], payload[25]]);
                    let timestamp_ms =
                        u32::from_le_bytes([payload[26], payload[27], payload[28], payload[29]]);
                    let (gx_dps, gy_dps, gz_dps, roll, pitch, yaw) = if payload.len() >= 54 {
                        (
                            f32::from_le_bytes([
                                payload[30],
                                payload[31],
                                payload[32],
                                payload[33],
                            ]),
                            f32::from_le_bytes([
                                payload[34],
                                payload[35],
                                payload[36],
                                payload[37],
                            ]),
                            f32::from_le_bytes([
                                payload[38],
                                payload[39],
                                payload[40],
                                payload[41],
                            ]),
                            f32::from_le_bytes([
                                payload[42],
                                payload[43],
                                payload[44],
                                payload[45],
                            ]),
                            f32::from_le_bytes([
                                payload[46],
                                payload[47],
                                payload[48],
                                payload[49],
                            ]),
                            f32::from_le_bytes([
                                payload[50],
                                payload[51],
                                payload[52],
                                payload[53],
                            ]),
                        )
                    } else {
                        (
                            gx as f32 / 131.0,
                            gy as f32 / 131.0,
                            gz as f32 / 131.0,
                            0.0,
                            0.0,
                            0.0,
                        )
                    };

                    Response::IMUData(IMUData {
                        ax,
                        ay,
                        az,
                        gx,
                        gy,
                        gz,
                        temp,
                        ax_g,
                        ay_g,
                        az_g,
                        gx_dps,
                        gy_dps,
                        gz_dps,
                        temp_c: temp as f32 / 340.0 + 36.53,
                        roll,
                        pitch,
                        yaw,
                        timestamp_ms,
                    })
                } else {
                    Response::Error(ERR_INVALID_PARAM)
                }
            }
            RSP_DIAGNOSTICS => {
                if payload.len() >= 60 {
                    let mut servo_currents = [0; 14];
                    for (index, current) in servo_currents.iter_mut().enumerate() {
                        let offset = 6 + index * 2;
                        *current = u16::from_be_bytes([payload[offset], payload[offset + 1]]);
                    }
                    let flags = payload[56];

                    Response::Diagnostics(Diagnostics {
                        uptime_ms: u32::from_be_bytes([
                            payload[0], payload[1], payload[2], payload[3],
                        ]),
                        free_ram: u16::from_be_bytes([payload[4], payload[5]]),
                        servo_currents,
                        total_current_ma: u16::from_be_bytes([payload[34], payload[35]]),
                        voltage_mv: u16::from_be_bytes([payload[36], payload[37]]),
                        temperature_c: i16::from_be_bytes([payload[38], payload[39]]),
                        packets_rx: u32::from_be_bytes([
                            payload[40],
                            payload[41],
                            payload[42],
                            payload[43],
                        ]),
                        packets_tx: u32::from_be_bytes([
                            payload[44],
                            payload[45],
                            payload[46],
                            payload[47],
                        ]),
                        crc_errors: u32::from_be_bytes([
                            payload[48],
                            payload[49],
                            payload[50],
                            payload[51],
                        ]),
                        timeout_errors: u32::from_be_bytes([
                            payload[52],
                            payload[53],
                            payload[54],
                            payload[55],
                        ]),
                        emergency_stop: flags & 0x01 != 0,
                        imu_connected: flags & 0x02 != 0,
                        firmware_version: [payload[57], payload[58], payload[59]],
                    })
                } else {
                    Response::Error(ERR_INVALID_PARAM)
                }
            }
            RSP_ERROR => Response::Error(payload.get(0).copied().unwrap_or(0)),
            _ => Response::Error(ERR_INVALID_CMD),
        }
    }

    fn clear_input_buffer(&mut self) -> Result<()> {
        if let Some(port) = &mut self.port {
            let mut buf = [0u8; 256];
            loop {
                match port.read(&mut buf) {
                    Ok(0) => break,
                    Ok(_) => continue,
                    Err(e) if e.kind() == std::io::ErrorKind::TimedOut => break,
                    Err(e) => return Err(e.into()),
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SerialPortInfo {
    pub port_name: String,
    pub port_type: String,
    pub usb_info: Option<UsbInfo>,
}

#[derive(Debug, Clone)]
pub struct UsbInfo {
    pub vid: u16,
    pub pid: u16,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial_number: Option<String>,
}

impl From<serialport::SerialPortInfo> for SerialPortInfo {
    fn from(info: serialport::SerialPortInfo) -> Self {
        let (port_type, usb_info) = match info.port_type {
            SerialPortType::UsbPort(usb) => (
                "USB".to_string(),
                Some(UsbInfo {
                    vid: usb.vid,
                    pid: usb.pid,
                    manufacturer: usb.manufacturer,
                    product: usb.product,
                    serial_number: usb.serial_number,
                }),
            ),
            SerialPortType::BluetoothPort => ("Bluetooth".to_string(), None),
            SerialPortType::PciPort => ("PCI".to_string(), None),
            SerialPortType::Unknown => ("Unknown".to_string(), None),
        };

        Self {
            port_name: info.port_name,
            port_type,
            usb_info,
        }
    }
}

fn error_to_str(err: u8) -> &'static str {
    match err {
        ERR_NONE => "No error",
        ERR_INVALID_CMD => "Invalid command",
        ERR_INVALID_PARAM => "Invalid parameter",
        ERR_CHECKSUM_FAIL => "Checksum failure",
        ERR_SERVO_INDEX => "Invalid servo index",
        ERR_ANGLE_OUT_OF_RANGE => "Angle out of range",
        ERR_PULSE_OUT_OF_RANGE => "Pulse out of range",
        ERR_IMU_NOT_FOUND => "IMU not found",
        ERR_IMU_INIT_FAIL => "IMU initialization failed",
        ERR_EEPROM_WRITE_FAIL => "EEPROM write failed",
        ERR_SAFETY_VIOLATION => "Safety violation",
        ERR_COMM_TIMEOUT => "Communication timeout",
        ERR_SEQUENCE_RUNNING => "Sequence already running",
        _ => "Unknown error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SerialConfig;

    #[test]
    fn diagnostics_parser_decodes_full_firmware_payload() {
        let manager = SerialManager::new(SerialConfig::default());
        let mut payload = [0; 60];
        payload[0..4].copy_from_slice(&1000_u32.to_be_bytes());
        payload[4..6].copy_from_slice(&2048_u16.to_be_bytes());
        for index in 0..14 {
            let offset = 6 + index * 2;
            payload[offset..offset + 2].copy_from_slice(&(100 + index as u16).to_be_bytes());
        }
        payload[34..36].copy_from_slice(&1500_u16.to_be_bytes());
        payload[36..38].copy_from_slice(&5000_u16.to_be_bytes());
        payload[38..40].copy_from_slice(&25_i16.to_be_bytes());
        payload[40..44].copy_from_slice(&11_u32.to_be_bytes());
        payload[44..48].copy_from_slice(&12_u32.to_be_bytes());
        payload[48..52].copy_from_slice(&13_u32.to_be_bytes());
        payload[52..56].copy_from_slice(&14_u32.to_be_bytes());
        payload[56] = 0x03;
        payload[57..60].copy_from_slice(&[1, 2, 3]);

        match manager.parse_response(RSP_DIAGNOSTICS, &payload) {
            Response::Diagnostics(data) => {
                assert_eq!(data.uptime_ms, 1000);
                assert_eq!(data.free_ram, 2048);
                assert_eq!(data.servo_currents[13], 113);
                assert_eq!(data.total_current_ma, 1500);
                assert_eq!(data.voltage_mv, 5000);
                assert_eq!(data.temperature_c, 25);
                assert_eq!(data.packets_rx, 11);
                assert_eq!(data.packets_tx, 12);
                assert_eq!(data.crc_errors, 13);
                assert_eq!(data.timeout_errors, 14);
                assert!(data.emergency_stop);
                assert!(data.imu_connected);
                assert_eq!(data.firmware_version, [1, 2, 3]);
            }
            _ => panic!("expected diagnostics response"),
        }
    }

    #[test]
    fn diagnostics_parser_rejects_truncated_payload() {
        let manager = SerialManager::new(SerialConfig::default());
        assert!(matches!(
            manager.parse_response(RSP_DIAGNOSTICS, &[0; 59]),
            Response::Error(ERR_INVALID_PARAM)
        ));
    }

    #[test]
    fn imu_parser_decodes_gyro_rates_and_orientation() {
        let manager = SerialManager::new(SerialConfig::default());
        let mut payload = [0; 54];
        payload[8..10].copy_from_slice(&131_i16.to_be_bytes());
        payload[14..18].copy_from_slice(&0.1_f32.to_le_bytes());
        payload[18..22].copy_from_slice(&0.2_f32.to_le_bytes());
        payload[22..26].copy_from_slice(&0.9_f32.to_le_bytes());
        payload[26..30].copy_from_slice(&1234_u32.to_le_bytes());
        for (index, value) in [1.0_f32, 2.0, 3.0, 10.0, -5.0, 20.0].iter().enumerate() {
            let offset = 30 + index * 4;
            payload[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }

        match manager.parse_response(RSP_IMU_DATA, &payload) {
            Response::IMUData(data) => {
                assert_eq!([data.gx_dps, data.gy_dps, data.gz_dps], [1.0, 2.0, 3.0]);
                assert_eq!([data.roll, data.pitch, data.yaw], [10.0, -5.0, 20.0]);
                assert_eq!(data.timestamp_ms, 1234);
            }
            _ => panic!("expected IMU response"),
        }
    }

    #[test]
    fn imu_parser_accepts_legacy_payload() {
        let manager = SerialManager::new(SerialConfig::default());
        let mut payload = [0; 30];
        payload[6..8].copy_from_slice(&131_i16.to_be_bytes());

        match manager.parse_response(RSP_IMU_DATA, &payload) {
            Response::IMUData(data) => {
                assert_eq!(data.gx_dps, 1.0);
                assert_eq!([data.roll, data.pitch, data.yaw], [0.0; 3]);
            }
            _ => panic!("expected legacy IMU response"),
        }
    }
}
