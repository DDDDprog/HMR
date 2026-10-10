// Serial protocol for communicating with Arduino firmware
// Protocol:
// - Commands: text-based, newline-terminated
// - Baud rate: 115200
// - Commands: ENABLE, DISABLE, SET <index> <angle>, GET <index>, HOME, MPU, STATUS, PING
// - Responses: OK, ERROR <message>, GET <index> <angle>, MPU <ax> <ay> <az> <gx> <gy> <gz>, STATUS, PONG

use anyhow::{anyhow, Result};
use std::io::{BufRead, BufReader, Write};

const BAUD_RATE: u32 = 115200;

/// Command types for serial communication
#[derive(Debug, Clone)]
pub enum Command {
    Enable,
    Disable,
    Set { index: usize, angle: i32 },
    Get { index: usize },
    Home,
    Mpu,
    Status,
    Ping,
}

impl Command {
    /// Convert command to serial string
    pub fn to_string(&self) -> String {
        match self {
            Command::Enable => "ENABLE".to_string(),
            Command::Disable => "DISABLE".to_string(),
            Command::Set { index, angle } => format!("SET {} {}", index, angle),
            Command::Get { index } => format!("GET {}", index),
            Command::Home => "HOME".to_string(),
            Command::Mpu => "MPU".to_string(),
            Command::Status => "STATUS".to_string(),
            Command::Ping => "PING".to_string(),
        }
    }
}

/// Response types from Arduino
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Response {
    Ok,
    Error(String),
    Get {
        index: usize,
        angle: i32,
    },
    Mpu {
        accel_x: i16,
        accel_y: i16,
        accel_z: i16,
        gyro_x: i16,
        gyro_y: i16,
        gyro_z: i16,
    },
    Status {
        enabled: bool,
        mpu_initialized: bool,
        angles: Vec<i32>,
    },
    Pong,
    Unknown(String),
}

impl Response {
    /// Parse a response string from Arduino
    pub fn parse(line: &str) -> Result<Response> {
        let line = line.trim();
        if line.is_empty() {
            return Err(anyhow!("Empty response"));
        }

        let parts: Vec<&str> = line.split_whitespace().collect();

        match parts.get(0).map(|s| *s) {
            Some("OK") => Ok(Response::Ok),
            Some("ERROR") => {
                let msg = parts[1..].join(" ");
                Ok(Response::Error(msg))
            }
            Some("GET") => {
                if parts.len() != 3 {
                    return Err(anyhow!("Invalid GET response format"));
                }
                let index: usize = parts[1].parse()?;
                let angle: i32 = parts[2].parse()?;
                Ok(Response::Get { index, angle })
            }
            Some("MPU") => {
                if parts.len() != 7 {
                    return Err(anyhow!("Invalid MPU response format"));
                }
                Ok(Response::Mpu {
                    accel_x: parts[1].parse()?,
                    accel_y: parts[2].parse()?,
                    accel_z: parts[3].parse()?,
                    gyro_x: parts[4].parse()?,
                    gyro_y: parts[5].parse()?,
                    gyro_z: parts[6].parse()?,
                })
            }
            Some("STATUS") => {
                let enabled = parts.iter().any(|p| p.starts_with("enabled=true"));
                let mpu_initialized = parts.iter().any(|p| p.starts_with("mpu=true"));

                let angles_part = parts
                    .iter()
                    .find(|p| p.starts_with("servos="))
                    .ok_or_else(|| anyhow!("Missing servos in STATUS"))?;

                let angles_str = angles_part
                    .strip_prefix("servos=")
                    .ok_or_else(|| anyhow!("Invalid servos format"))?;

                let angles: Vec<i32> = angles_str
                    .split(',')
                    .map(|s| s.parse::<i32>())
                    .collect::<Result<_, _>>()?;

                Ok(Response::Status {
                    enabled,
                    mpu_initialized,
                    angles,
                })
            }
            Some("PONG") => Ok(Response::Pong),
            Some("READY") => Ok(Response::Unknown("READY".to_string())),
            _ => Ok(Response::Unknown(line.to_string())),
        }
    }
}

/// Serial port wrapper for Arduino communication
pub struct RobotSerial {
    port: Box<dyn serialport::SerialPort>,
}

impl RobotSerial {
    /// Open serial connection to Arduino
    pub fn open(port_name: &str) -> Result<Self> {
        let port = serialport::new(port_name, BAUD_RATE)
            .timeout(std::time::Duration::from_millis(100))
            .open()?;

        Ok(Self { port })
    }

    /// Send a command and wait for response
    pub fn send_command(&mut self, cmd: &Command) -> Result<Response> {
        let cmd_str = cmd.to_string();
        self.port.write_all(cmd_str.as_bytes())?;
        self.port.write_all(b"\n")?;

        let mut reader = BufReader::new(&mut self.port);
        let mut line = String::new();
        reader.read_line(&mut line)?;

        Response::parse(&line)
    }

    /// Send command without waiting for response
    #[allow(dead_code)]
    pub fn send_command_no_wait(&mut self, cmd: &Command) -> Result<()> {
        let cmd_str = cmd.to_string();
        self.port.write_all(cmd_str.as_bytes())?;
        self.port.write_all(b"\n")?;
        Ok(())
    }

    /// Read a response (after sending command without wait)
    #[allow(dead_code)]
    pub fn read_response(&mut self) -> Result<Response> {
        let mut reader = BufReader::new(&mut self.port);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        Response::parse(&line)
    }

    /// Check if Arduino is ready
    pub fn wait_for_ready(&mut self, timeout_ms: u64) -> Result<bool> {
        let start = std::time::Instant::now();
        let mut reader = BufReader::new(&mut self.port);

        while start.elapsed().as_millis() < timeout_ms as u128 {
            let mut line = String::new();
            if reader.read_line(&mut line).is_ok() {
                if line.trim() == "READY" {
                    return Ok(true);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_to_string() {
        assert_eq!(Command::Enable.to_string(), "ENABLE");
        assert_eq!(Command::Disable.to_string(), "DISABLE");
        assert_eq!(
            Command::Set {
                index: 0,
                angle: 45
            }
            .to_string(),
            "SET 0 45"
        );
        assert_eq!(Command::Get { index: 5 }.to_string(), "GET 5");
        assert_eq!(Command::Home.to_string(), "HOME");
        assert_eq!(Command::Mpu.to_string(), "MPU");
        assert_eq!(Command::Status.to_string(), "STATUS");
        assert_eq!(Command::Ping.to_string(), "PING");
    }

    #[test]
    fn test_response_parse_ok() {
        let resp = Response::parse("OK").unwrap();
        assert!(matches!(resp, Response::Ok));
    }

    #[test]
    fn test_response_parse_error() {
        let resp = Response::parse("ERROR invalid servo index").unwrap();
        assert!(matches!(resp, Response::Error(_)));
    }

    #[test]
    fn test_response_parse_get() {
        let resp = Response::parse("GET 0 45").unwrap();
        match resp {
            Response::Get { index, angle } => {
                assert_eq!(index, 0);
                assert_eq!(angle, 45);
            }
            _ => panic!("Expected GET response"),
        }
    }

    #[test]
    fn test_response_parse_mpu() {
        let resp = Response::parse("MPU 100 200 300 10 20 30").unwrap();
        match resp {
            Response::Mpu {
                accel_x,
                accel_y,
                accel_z,
                gyro_x,
                gyro_y,
                gyro_z,
            } => {
                assert_eq!(accel_x, 100);
                assert_eq!(accel_y, 200);
                assert_eq!(accel_z, 300);
                assert_eq!(gyro_x, 10);
                assert_eq!(gyro_y, 20);
                assert_eq!(gyro_z, 30);
            }
            _ => panic!("Expected MPU response"),
        }
    }

    #[test]
    fn test_response_parse_status() {
        let resp =
            Response::parse("STATUS enabled=true mpu=true servos=0,0,0,0,0,0,0,0,0,0,0,0,0,0")
                .unwrap();
        match resp {
            Response::Status {
                enabled,
                mpu_initialized,
                angles,
            } => {
                assert!(enabled);
                assert!(mpu_initialized);
                assert_eq!(angles.len(), 14);
            }
            _ => panic!("Expected STATUS response"),
        }
    }

    #[test]
    fn test_response_parse_pong() {
        let resp = Response::parse("PONG").unwrap();
        assert!(matches!(resp, Response::Pong));
    }

    #[test]
    fn test_response_parse_ready() {
        let resp = Response::parse("READY").unwrap();
        assert!(matches!(resp, Response::Unknown(_)));
    }

    #[test]
    fn test_response_parse_invalid_get() {
        assert!(Response::parse("GET 0").is_err());
    }

    #[test]
    fn test_response_parse_invalid_mpu() {
        assert!(Response::parse("MPU 1 2 3").is_err());
    }
}
