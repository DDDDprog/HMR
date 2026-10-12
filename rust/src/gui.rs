use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use eframe::egui;

use crate::config::{AppConfig, SerialTransport};
use crate::serial::{Diagnostics, IMUData, SerialManager, ServoState};

fn default_port_for_transport(transport: SerialTransport) -> String {
    if cfg!(target_os = "linux") {
        match transport {
            SerialTransport::WiredUsb => "/dev/ttyACM0".to_string(),
            SerialTransport::WirelessBluetooth => "/dev/rfcomm0".to_string(),
        }
    } else if cfg!(target_os = "windows") {
        match transport {
            SerialTransport::WiredUsb => String::new(),
            SerialTransport::WirelessBluetooth => "COM3".to_string(),
        }
    } else {
        String::new()
    }
}

fn baud_for_transport(transport: SerialTransport) -> u32 {
    match transport {
        SerialTransport::WiredUsb => 115_200,
        SerialTransport::WirelessBluetooth => 9_600,
    }
}

fn port_transport_warning(transport: SerialTransport, port: &str) -> Option<&'static str> {
    if !cfg!(target_os = "linux") {
        return None;
    }

    match transport {
        SerialTransport::WirelessBluetooth
            if port.starts_with("/dev/ttyACM") || port.starts_with("/dev/ttyUSB") =>
        {
            Some("This is a USB port. Choose Wired USB or select /dev/rfcomm0.")
        }
        SerialTransport::WiredUsb if port.starts_with("/dev/rfcomm") => {
            Some("This is an RFCOMM port. Choose Wireless Bluetooth or select a USB port.")
        }
        _ => None,
    }
}

fn connect_and_verify(
    serial: &mut SerialManager,
    port: String,
    baud_rate: u32,
) -> anyhow::Result<bool> {
    serial.set_port(port);
    serial.set_baud_rate(baud_rate);
    serial.connect()?;
    match serial.send_ping() {
        Ok(true) => Ok(true),
        Ok(false) => {
            serial.disconnect();
            Ok(false)
        }
        Err(error) => {
            serial.disconnect();
            Err(error)
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum JointGroup {
    All,
    Legs,
    Arms,
    Hands,
    Head,
}

impl JointGroup {
    fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Legs => "Legs",
            Self::Arms => "Arms",
            Self::Hands => "Hands",
            Self::Head => "Head",
        }
    }

    fn indices(self) -> &'static [usize] {
        const ALL: [usize; 14] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
        const LEGS: [usize; 6] = [0, 1, 2, 3, 4, 5];
        const ARMS: [usize; 4] = [6, 7, 9, 10];
        const HANDS: [usize; 2] = [8, 11];
        const HEAD: [usize; 2] = [12, 13];

        match self {
            Self::All => &ALL,
            Self::Legs => &LEGS,
            Self::Arms => &ARMS,
            Self::Hands => &HANDS,
            Self::Head => &HEAD,
        }
    }
}

pub struct HumanoidRobotApp {
    config: AppConfig,
    serial_manager: Arc<Mutex<SerialManager>>,
    available_ports: Vec<String>,
    robot_online: Option<bool>,
    joint_group: JointGroup,
    selected_servo: usize,
    servo_angle: f32,
    servo_pulse_us: u16,
    servo_state: Option<ServoState>,
    imu_data: Option<IMUData>,
    diagnostics: Option<Diagnostics>,
    imu_streaming: bool,
    last_ping: Instant,
    status: String,
}

impl HumanoidRobotApp {
    pub fn new(config: AppConfig, serial_manager: Arc<Mutex<SerialManager>>) -> Self {
        let available_ports = SerialManager::list_ports()
            .unwrap_or_default()
            .into_iter()
            .map(|port| port.port_name)
            .collect();

        let (robot_online, status) = if config.serial.auto_connect {
            let mut serial = serial_manager.lock().unwrap();
            match connect_and_verify(
                &mut serial,
                config.serial.port.clone(),
                config.serial.baud_rate,
            ) {
                Ok(true) => (Some(true), String::new()),
                Ok(false) => (
                    Some(false),
                    "Port opened, but robot did not answer ping".to_string(),
                ),
                Err(err) => (Some(false), format!("Auto-connect failed: {err:#}")),
            }
        } else {
            (None, String::new())
        };

        Self {
            config,
            serial_manager,
            available_ports,
            robot_online,
            joint_group: JointGroup::All,
            selected_servo: 0,
            servo_angle: 90.0,
            servo_pulse_us: 1500,
            servo_state: None,
            imu_data: None,
            diagnostics: None,
            imu_streaming: false,
            last_ping: Instant::now(),
            status,
        }
    }
}

impl eframe::App for HumanoidRobotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let connected = self.serial_manager.lock().unwrap().is_connected();

        if connected
            && self.robot_online == Some(true)
            && self.last_ping.elapsed() >= Duration::from_secs(1)
        {
            self.last_ping = Instant::now();
            match self.serial_manager.lock().unwrap().send_ping() {
                Ok(true) => {}
                Ok(false) => {
                    self.robot_online = Some(false);
                    self.status =
                        "Robot heartbeat missed; check the selected link and power".to_string();
                }
                Err(err) => {
                    self.robot_online = Some(false);
                    self.status = format!("Robot heartbeat failed: {err:#}");
                }
            }
        }

        if self.imu_streaming {
            ctx.request_repaint_after(Duration::from_millis(40));
            if connected {
                match self.serial_manager.lock().unwrap().poll_stream() {
                    Ok(responses) => {
                        for response in responses {
                            if let crate::serial::Response::IMUData(data) = response {
                                self.imu_data = Some(data);
                            }
                        }
                    }
                    Err(err) => {
                        self.imu_streaming = false;
                        self.robot_online = Some(false);
                        self.status = format!("IMU stream failed: {err:#}");
                    }
                }
            } else {
                self.imu_streaming = false;
                self.robot_online = None;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Humanoid Robot Control");
            ui.separator();

            let mut transport_changed = false;
            ui.horizontal(|ui| {
                ui.label("Link mode:");
                ui.add_enabled_ui(!connected, |ui| {
                    transport_changed |= ui
                        .selectable_value(
                            &mut self.config.serial.transport,
                            SerialTransport::WiredUsb,
                            "Wired USB",
                        )
                        .changed();
                    transport_changed |= ui
                        .selectable_value(
                            &mut self.config.serial.transport,
                            SerialTransport::WirelessBluetooth,
                            "Wireless Bluetooth",
                        )
                        .changed();
                });
            });

            if transport_changed {
                self.config.serial.port = default_port_for_transport(self.config.serial.transport);
                self.config.serial.baud_rate = baud_for_transport(self.config.serial.transport);
                self.robot_online = None;
                self.status = match self.config.serial.transport {
                    SerialTransport::WiredUsb => {
                        "Use firmware built with USE_BLUETOOTH_SERIAL 0".to_string()
                    }
                    SerialTransport::WirelessBluetooth => {
                        "Use firmware built with USE_BLUETOOTH_SERIAL 1".to_string()
                    }
                };
            }

            if connected {
                ui.weak("Disconnect before changing link mode.");
            }
            ui.small(match self.config.serial.transport {
                SerialTransport::WiredUsb => {
                    "Mega USB serial: 115200 baud; firmware must be built for USB mode."
                }
                SerialTransport::WirelessBluetooth => {
                    "Bluetooth SPP: 9600 baud; pair/bind the module and use Serial1 firmware."
                }
            });

            ui.horizontal(|ui| {
                ui.label("Port:");
                ui.add_enabled(
                    !connected,
                    egui::TextEdit::singleline(&mut self.config.serial.port)
                        .desired_width(220.0)
                        .hint_text("Select a detected port or enter it"),
                );
                if ui.add_enabled(!connected, egui::Button::new("Refresh ports")).clicked() {
                    match SerialManager::list_ports() {
                        Ok(ports) => {
                            self.available_ports = ports
                                .into_iter()
                                .map(|port| port.port_name)
                                .collect();
                        }
                        Err(err) => {
                            self.status = format!("Port scan error: {err:#}");
                        }
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Detected:");
                ui.add_enabled_ui(!connected, |ui| {
                    egui::ComboBox::from_id_salt("serial_port_picker")
                        .selected_text(if self.config.serial.port.is_empty() {
                            "Select port"
                        } else {
                            &self.config.serial.port
                        })
                        .show_ui(ui, |ui| {
                            for port in &self.available_ports {
                                ui.selectable_value(
                                    &mut self.config.serial.port,
                                    port.clone(),
                                    port,
                                );
                            }
                        });
                });
                if self.available_ports.is_empty() {
                    ui.weak("No ports detected; enter the device path after connecting it.");
                }
            });
            let port_warning =
                port_transport_warning(self.config.serial.transport, &self.config.serial.port);
            if let Some(warning) = port_warning {
                ui.colored_label(egui::Color32::from_rgb(205, 145, 35), warning);
            }

            ui.horizontal(|ui| {
                ui.label("Baud rate:");
                ui.add_enabled(
                    !connected,
                    egui::DragValue::new(&mut self.config.serial.baud_rate)
                        .range(1200..=1_000_000)
                        .speed(100)
                        .suffix(" baud"),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Serial port:");
                let (label, color) = if connected {
                    ("OPEN", egui::Color32::from_rgb(40, 150, 80))
                } else {
                    ("CLOSED", egui::Color32::from_rgb(190, 65, 55))
                };
                ui.colored_label(color, egui::RichText::new(label).strong());
                if !self.status.is_empty() {
                    ui.small(&self.status);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Robot:");
                let (label, color) = match self.robot_online {
                    Some(true) => ("RESPONDING", egui::Color32::from_rgb(40, 150, 80)),
                    Some(false) => ("NO RESPONSE", egui::Color32::from_rgb(190, 65, 55)),
                    None => ("NOT CHECKED", ui.visuals().widgets.noninteractive.fg_stroke.color),
                };
                ui.colored_label(color, egui::RichText::new(label).strong());
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        !connected
                            && !self.config.serial.port.trim().is_empty()
                            && port_warning.is_none(),
                        egui::Button::new("Connect"),
                    )
                    .clicked()
                {
                    let result = {
                        let mut serial = self.serial_manager.lock().unwrap();
                        connect_and_verify(
                            &mut serial,
                            self.config.serial.port.clone(),
                            self.config.serial.baud_rate,
                        )
                    };
                    match result {
                        Ok(true) => {
                            self.robot_online = Some(true);
                            self.last_ping = Instant::now();
                            self.status.clear();
                        }
                        Ok(false) => {
                            self.robot_online = Some(false);
                            self.status = match self.config.serial.transport {
                                SerialTransport::WiredUsb => "USB port opened, but robot did not answer. Check firmware was built with USE_BLUETOOTH_SERIAL 0, baud is 115200, and the Mega is powered.".to_string(),
                                SerialTransport::WirelessBluetooth => "Bluetooth port opened, but robot did not answer. Check HC-05 pairing/RFCOMM, firmware USE_BLUETOOTH_SERIAL 1, 9600 baud, UART wiring, and Mega power.".to_string(),
                            };
                        }
                        Err(err) => {
                            self.robot_online = Some(false);
                            self.status = format!("Connection/check error: {err:#}");
                        }
                    }
                }

                if ui.add_enabled(connected, egui::Button::new("Check robot")).clicked() {
                    let result = self.serial_manager.lock().unwrap().send_ping();
                    match result {
                        Ok(true) => {
                            self.robot_online = Some(true);
                            self.last_ping = Instant::now();
                            self.status.clear();
                        }
                        Ok(false) => {
                            self.robot_online = Some(false);
                            self.status = "No ping response; check robot power and wiring".to_string();
                        }
                        Err(err) => {
                            self.robot_online = Some(false);
                            self.status = format!("Robot check error: {err:#}");
                        }
                    }
                }

                if ui.add_enabled(connected, egui::Button::new("Disconnect")).clicked() {
                    let mut serial = self.serial_manager.lock().unwrap();
                    if self.imu_streaming {
                        let _ = serial.stop_stream();
                    }
                    serial.disconnect();
                    self.imu_streaming = false;
                    self.robot_online = None;
                    self.servo_state = None;
                    self.imu_data = None;
                    self.diagnostics = None;
                    self.status = "Disconnected".to_string();
                }
                if ui.button("Save app settings").clicked() {
                    self.status = match self.config.save() {
                        Ok(()) => "App settings saved".to_string(),
                        Err(err) => format!("App settings save failed: {err}"),
                    };
                }
            });

            ui.separator();

            let robot_ready = connected && self.robot_online == Some(true);
            ui.horizontal(|ui| {
                ui.label("Live IMU:");
                if !self.imu_streaming {
                    if ui
                        .add_enabled(robot_ready, egui::Button::new("Start stream"))
                        .clicked()
                    {
                        let rate_ms = self.config.imu.stream_rate_ms.max(20);
                        let result = self
                            .serial_manager
                            .lock()
                            .unwrap()
                            .start_stream(0x01, rate_ms);
                        match result {
                            Ok(true) => {
                                self.imu_streaming = true;
                                self.status = format!("IMU stream running at {rate_ms} ms");
                            }
                            Ok(false) => self.status = "IMU stream was not acknowledged".to_string(),
                            Err(err) => self.status = format!("Could not start IMU stream: {err:#}"),
                        }
                    }
                } else if ui.button("Stop stream").clicked() {
                    let result = self.serial_manager.lock().unwrap().stop_stream();
                    self.imu_streaming = false;
                    self.status = match result {
                        Ok(true) => "IMU stream stopped".to_string(),
                        Ok(false) => "Stream stop was not acknowledged".to_string(),
                        Err(err) => format!("Could not stop IMU stream: {err:#}"),
                    };
                }
                if self.imu_streaming {
                    ui.colored_label(egui::Color32::from_rgb(40, 150, 80), "LIVE");
                    ui.label("other serial actions paused during stream");
                }
            });

            ui.add_enabled_ui(robot_ready && !self.imu_streaming, |ui| {
                ui.columns(2, |columns| {
                    columns[0].heading("Servo control");
                    let previous_group = self.joint_group;
                    columns[0].horizontal(|ui| {
                        ui.label("Group:");
                        for group in [
                            JointGroup::Legs,
                            JointGroup::Arms,
                            JointGroup::Hands,
                            JointGroup::Head,
                            JointGroup::All,
                        ] {
                            ui.selectable_value(
                                &mut self.joint_group,
                                group,
                                group.label(),
                            );
                        }
                    });
                    if self.joint_group != previous_group {
                        self.selected_servo = self.joint_group.indices()[0];
                        let limit = &self.config.servo_limits.limits[self.selected_servo];
                        self.servo_angle = (limit.min_angle + limit.max_angle) / 2.0;
                        self.servo_pulse_us = limit.center_pulse;
                        self.servo_state = None;
                    }
                    let joint_indices = self.joint_group.indices().to_vec();
                    columns[0].horizontal(|ui| {
                        ui.label("Joint:");
                        egui::ComboBox::from_id_salt("servo_picker")
                            .selected_text(&self.config.servo_limits.limits[self.selected_servo].name)
                            .show_ui(ui, |ui| {
                                for index in &joint_indices {
                                    let limit = &self.config.servo_limits.limits[*index];
                                    ui.selectable_value(
                                        &mut self.selected_servo,
                                        *index,
                                        &limit.name,
                                    );
                                }
                            });
                    });

                    let selected_limit = &mut self.config.servo_limits.limits[self.selected_servo];
                    columns[0].add(
                        egui::Slider::new(
                            &mut self.servo_angle,
                            selected_limit.min_angle..=selected_limit.max_angle,
                        )
                        .text("Angle")
                        .suffix(" deg"),
                    );
                    columns[0].horizontal(|ui| {
                        if ui.button("Apply angle").clicked() {
                            let result = self
                                .serial_manager
                                .lock()
                                .unwrap()
                                .set_servo_angle(self.selected_servo as u8, self.servo_angle);
                            self.status = match result {
                                Ok(true) => "Servo angle command acknowledged".to_string(),
                                Ok(false) => "No acknowledgement for servo command".to_string(),
                                Err(err) => format!("Servo command failed: {err:#}"),
                            };
                        }
                        if ui.button("Read state").clicked() {
                            match self
                                .serial_manager
                                .lock()
                                .unwrap()
                                .get_servo_state(self.selected_servo as u8)
                            {
                                Ok(Some(state)) => {
                                    self.servo_angle = state.angle;
                                    self.servo_pulse_us = state.pulse_us;
                                    self.servo_state = Some(state);
                                    self.status = "Servo state received".to_string();
                                }
                                Ok(None) => self.status = "No servo state received".to_string(),
                                Err(err) => self.status = format!("Servo read failed: {err:#}"),
                            }
                        }
                    });

                    columns[0].horizontal(|ui| {
                        ui.label("Pulse:");
                        ui.add(egui::DragValue::new(&mut self.servo_pulse_us).range(400..=2600).suffix(" us"));
                        if ui.button("Apply pulse").clicked() {
                            let result = self
                                .serial_manager
                                .lock()
                                .unwrap()
                                .set_servo_pulse(self.selected_servo as u8, self.servo_pulse_us);
                            self.status = match result {
                                Ok(true) => "Servo pulse command acknowledged".to_string(),
                                Ok(false) => "No acknowledgement for pulse command".to_string(),
                                Err(err) => format!("Servo pulse command failed: {err:#}"),
                            };
                        }
                    });

                    if let Some(state) = &self.servo_state {
                        columns[0].label(format!(
                            "Joint {}: {:.1} deg, {} us, {}",
                            state.index + 1,
                            state.angle,
                            state.pulse_us,
                            if state.attached { "attached" } else { "detached" }
                        ));
                    }

                    columns[0].separator();
                    columns[0].heading("Joint limits");
                    columns[0].add(
                        egui::Slider::new(&mut selected_limit.min_angle, 0.0..=180.0)
                            .text("Minimum angle")
                            .suffix(" deg"),
                    );
                    columns[0].add(
                        egui::Slider::new(&mut selected_limit.max_angle, 0.0..=180.0)
                            .text("Maximum angle")
                            .suffix(" deg"),
                    );
                    columns[0].horizontal(|ui| {
                        if ui.button("Read joint limits").clicked() {
                            match self
                                .serial_manager
                                .lock()
                                .unwrap()
                                .get_servo_limits(self.selected_servo as u8)
                            {
                                Ok(Some(limits)) => {
                                    selected_limit.min_angle = limits.min_angle;
                                    selected_limit.max_angle = limits.max_angle;
                                    selected_limit.min_pulse = limits.min_pulse;
                                    selected_limit.max_pulse = limits.max_pulse;
                                    selected_limit.center_pulse = limits.center_pulse;
                                    self.status = "Joint limits received from robot".to_string();
                                }
                                Ok(None) => self.status = "No joint limits received".to_string(),
                                Err(err) => self.status = format!("Limit read failed: {err:#}"),
                            }
                        }
                        ui.label("Pulse min/max/center (us):");
                        ui.add(egui::DragValue::new(&mut selected_limit.min_pulse).range(400..=2600));
                        ui.add(egui::DragValue::new(&mut selected_limit.max_pulse).range(400..=2600));
                        ui.add(egui::DragValue::new(&mut selected_limit.center_pulse).range(400..=2600));
                    });
                    if columns[0].button("Apply joint limits").clicked() {
                        if selected_limit.min_angle >= selected_limit.max_angle
                            || selected_limit.min_pulse >= selected_limit.max_pulse
                            || !(selected_limit.min_pulse..=selected_limit.max_pulse)
                                .contains(&selected_limit.center_pulse)
                        {
                            self.status = "Limits invalid: min must be below max and center within pulse range".to_string();
                        } else {
                            let limits = crate::serial::ServoLimits {
                                index: self.selected_servo as u8,
                                min_angle: selected_limit.min_angle,
                                max_angle: selected_limit.max_angle,
                                min_pulse: selected_limit.min_pulse,
                                max_pulse: selected_limit.max_pulse,
                                center_pulse: selected_limit.center_pulse,
                            };
                            let result = self
                                .serial_manager
                                .lock()
                                .unwrap()
                                .set_servo_limits(self.selected_servo as u8, &limits);
                            self.status = match result {
                                Ok(true) => "Joint limits acknowledged".to_string(),
                                Ok(false) => "No acknowledgement for joint limits".to_string(),
                                Err(err) => format!("Limit update failed: {err:#}"),
                            };
                        }
                    }

                    columns[1].heading("Sensors and diagnostics");
                    columns[1].horizontal(|ui| {
                        if ui.button("Read IMU").clicked() {
                            match self.serial_manager.lock().unwrap().get_imu_data() {
                                Ok(Some(data)) => {
                                    self.imu_data = Some(data);
                                    self.status = "IMU data received".to_string();
                                }
                                Ok(None) => self.status = "IMU unavailable or no response".to_string(),
                                Err(err) => self.status = format!("IMU read failed: {err:#}"),
                            }
                        }
                        if ui.button("Calibrate IMU").clicked() {
                            let result = self.serial_manager.lock().unwrap().calibrate_imu();
                            self.status = match result {
                                Ok(true) => "IMU calibration saved".to_string(),
                                Ok(false) => "IMU calibration not acknowledged".to_string(),
                                Err(err) => format!("IMU calibration failed: {err:#}"),
                            };
                        }
                    });
                    if let Some(data) = &self.imu_data {
                        let tilt = data.roll.abs().max(data.pitch.abs());
                        let (tilt_label, tilt_color) = if tilt >= 30.0 {
                            ("HIGH TILT", egui::Color32::from_rgb(190, 65, 55))
                        } else if tilt >= 15.0 {
                            ("TILT WARNING", egui::Color32::from_rgb(205, 145, 35))
                        } else {
                            ("LEVEL", egui::Color32::from_rgb(40, 150, 80))
                        };
                        columns[1].horizontal(|ui| {
                            ui.label("Tilt monitor (advisory):");
                            ui.colored_label(tilt_color, egui::RichText::new(tilt_label).strong());
                        });
                        columns[1].add(
                            egui::ProgressBar::new((tilt / 45.0).clamp(0.0, 1.0))
                                .text(format!("max |roll/pitch|: {tilt:.1} deg")),
                        );
                        columns[1].label(format!(
                            "Accel (g): X {:.3}  Y {:.3}  Z {:.3}",
                            data.ax_g, data.ay_g, data.az_g
                        ));
                        columns[1].label(format!(
                            "Gyro: X {:.1}  Y {:.1}  Z {:.1} deg/s",
                            data.gx_dps, data.gy_dps, data.gz_dps
                        ));
                        columns[1].label(format!(
                            "Orientation: roll {:.1}  pitch {:.1}  yaw {:.1} deg  |  {:.1} C",
                            data.roll, data.pitch, data.yaw, data.temp_c
                        ));
                        columns[1].small("Tilt thresholds are advisory only; no automatic balance correction is active.");
                    }

                    columns[1].separator();
                    if columns[1].button("Read diagnostics").clicked() {
                        match self.serial_manager.lock().unwrap().get_diagnostics() {
                            Ok(Some(data)) => {
                                self.diagnostics = Some(data);
                                self.status = "Diagnostics received".to_string();
                            }
                            Ok(None) => self.status = "No diagnostics received".to_string(),
                            Err(err) => self.status = format!("Diagnostics failed: {err:#}"),
                        }
                    }
                    if let Some(data) = &self.diagnostics {
                        columns[1].label(format!("Firmware: {}.{}.{}", data.firmware_version[0], data.firmware_version[1], data.firmware_version[2]));
                        columns[1].label(format!("Uptime: {} s  |  Free RAM: {} bytes", data.uptime_ms / 1000, data.free_ram));
                        columns[1].label(format!("Voltage: {} mV  |  Estimated current: {} mA", data.voltage_mv, data.total_current_ma));
                        columns[1].label(format!("Temperature: {} C", data.temperature_c));
                        columns[1].label(format!("IMU: {}  |  Emergency stop: {}", if data.imu_connected { "ready" } else { "not found" }, if data.emergency_stop { "active" } else { "clear" }));
                        columns[1].label(format!("Packets RX/TX: {}/{}  |  CRC errors: {}  |  Timeouts: {}", data.packets_rx, data.packets_tx, data.crc_errors, data.timeout_errors));
                        columns[1].collapsing("Estimated current by joint", |ui| {
                            for (index, current) in data.servo_currents.iter().enumerate() {
                                ui.label(format!("{}: {} mA", self.config.servo_limits.limits[index].name, current));
                            }
                        });
                    }

                    columns[1].separator();
                    columns[1].horizontal(|ui| {
                        if ui.button("Reset servos").clicked() {
                            let result = self.serial_manager.lock().unwrap().reset_servos();
                            self.status = match result {
                                Ok(true) => "Servos reset to defaults".to_string(),
                                Ok(false) => "Servo reset was not acknowledged".to_string(),
                                Err(err) => format!("Servo reset failed: {err:#}"),
                            };
                        }
                        if ui.button("Save robot config").clicked() {
                            let result = self.serial_manager.lock().unwrap().save_config();
                            self.status = match result {
                                Ok(true) => "Robot config saved to EEPROM".to_string(),
                                Ok(false) => "Robot config save not acknowledged".to_string(),
                                Err(err) => format!("Robot config save failed: {err:#}"),
                            };
                        }
                        if ui.button("Load robot config").clicked() {
                            let result = self.serial_manager.lock().unwrap().load_config();
                            self.status = match result {
                                Ok(true) => "Robot config loaded from EEPROM; read limits to refresh this view".to_string(),
                                Ok(false) => "Robot config load not acknowledged".to_string(),
                                Err(err) => format!("Robot config load failed: {err:#}"),
                            };
                        }
                    });
                });
            });

            ui.separator();
            ui.horizontal(|ui| {
                let stop = egui::Button::new("EMERGENCY STOP")
                    .fill(egui::Color32::from_rgb(175, 45, 40));
                if ui.add_enabled(robot_ready, stop).clicked() {
                    let result = {
                        let mut serial = self.serial_manager.lock().unwrap();
                        if self.imu_streaming {
                            let _ = serial.stop_stream();
                            self.imu_streaming = false;
                        }
                        serial.emergency_stop()
                    };
                    self.status = match result {
                        Ok(true) => "Emergency stop command acknowledged".to_string(),
                        Ok(false) => "Emergency stop was not acknowledged".to_string(),
                        Err(err) => format!("Emergency stop failed: {err:#}"),
                    };
                }
                if !self.status.is_empty() {
                    ui.label(&self.status);
                }
            });
        });
    }
}
