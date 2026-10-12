use dirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub serial: SerialConfig,
    pub gui: GuiConfig,
    pub servo_limits: ServoLimitsConfig,
    pub imu: ImuConfig,
    pub camera: CameraConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialConfig {
    #[serde(default)]
    pub transport: SerialTransport,
    pub port: String,
    pub baud_rate: u32,
    pub timeout_ms: u64,
    pub auto_connect: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SerialTransport {
    WiredUsb,
    WirelessBluetooth,
}

impl Default for SerialTransport {
    fn default() -> Self {
        Self::WiredUsb
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiConfig {
    pub theme: String,
    pub show_debug: bool,
    pub stream_rate_ms: u16,
    pub window_width: f32,
    pub window_height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServoLimitsConfig {
    pub limits: [ServoLimit; 14],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServoLimit {
    pub name: String,
    pub min_angle: f32,
    pub max_angle: f32,
    pub min_pulse: u16,
    pub max_pulse: u16,
    pub center_pulse: u16,
    pub reversed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImuConfig {
    pub enable_streaming: bool,
    pub stream_rate_ms: u16,
    pub show_orientation: bool,
    pub calibration_samples: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraConfig {
    pub device_index: usize,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            serial: SerialConfig::default(),
            gui: GuiConfig::default(),
            servo_limits: ServoLimitsConfig::default(),
            imu: ImuConfig::default(),
            camera: CameraConfig::default(),
        }
    }
}

impl Default for SerialConfig {
    fn default() -> Self {
        let (transport, port, baud_rate) = if cfg!(target_os = "linux") {
            (
                SerialTransport::WiredUsb,
                "/dev/ttyACM0".to_string(),
                115_200,
            )
        } else if cfg!(target_os = "windows") {
            (SerialTransport::WiredUsb, String::new(), 115_200)
        } else {
            (SerialTransport::WiredUsb, String::new(), 115_200)
        };

        Self {
            transport,
            port,
            baud_rate,
            timeout_ms: 1000,
            auto_connect: false,
        }
    }
}

impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            show_debug: false,
            stream_rate_ms: 100,
            window_width: 1200.0,
            window_height: 800.0,
        }
    }
}

impl Default for ServoLimitsConfig {
    fn default() -> Self {
        let names = [
            "Left Hip",
            "Left Knee",
            "Left Ankle",
            "Right Hip",
            "Right Knee",
            "Right Ankle",
            "Left Shoulder",
            "Left Elbow",
            "Left Claw",
            "Right Shoulder",
            "Right Elbow",
            "Right Claw",
            "Head Pan",
            "Head Tilt",
        ];

        let mut limits = std::array::from_fn(|_| ServoLimit::default());
        for (i, name) in names.iter().enumerate() {
            limits[i] = ServoLimit {
                name: name.to_string(),
                min_angle: 0.0,
                max_angle: 180.0,
                min_pulse: 500,
                max_pulse: 2500,
                center_pulse: 1500,
                reversed: false,
            };
        }

        Self { limits }
    }
}

impl Default for ServoLimit {
    fn default() -> Self {
        Self {
            name: "Servo".to_string(),
            min_angle: 0.0,
            max_angle: 180.0,
            min_pulse: 500,
            max_pulse: 2500,
            center_pulse: 1500,
            reversed: false,
        }
    }
}

impl Default for ImuConfig {
    fn default() -> Self {
        Self {
            enable_streaming: true,
            stream_rate_ms: 100,
            show_orientation: true,
            calibration_samples: 1000,
        }
    }
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            device_index: 0,
            width: 640,
            height: 480,
            fps: 30,
            enabled: true,
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: AppConfig = ron::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("Could not find config directory")?
            .join("humanoid-robot-gui");
        Ok(config_dir.join("config.ron"))
    }
}

#[cfg(test)]
mod tests {
    use super::{SerialConfig, SerialTransport};

    #[test]
    fn legacy_serial_config_defaults_transport() {
        let config: SerialConfig = ron::from_str(
            "(port: \"/dev/ttyACM0\", baud_rate: 115200, timeout_ms: 1000, auto_connect: false)",
        )
        .expect("legacy serial config should deserialize");

        assert_eq!(config.transport, SerialTransport::default());
        assert_eq!(config.port, "/dev/ttyACM0");
    }

    #[test]
    fn serial_transport_round_trips() {
        let config = SerialConfig {
            transport: SerialTransport::WiredUsb,
            ..SerialConfig::default()
        };
        let serialized = ron::to_string(&config).expect("serial config should serialize");
        let restored: SerialConfig =
            ron::from_str(&serialized).expect("serial config should deserialize");

        assert_eq!(restored.transport, SerialTransport::WiredUsb);
    }

    #[test]
    fn default_baud_matches_default_transport() {
        let config = SerialConfig::default();
        let expected_baud = match config.transport {
            SerialTransport::WiredUsb => 115_200,
            SerialTransport::WirelessBluetooth => 9_600,
        };

        assert_eq!(config.baud_rate, expected_baud);
    }
}
