use anyhow::Result;
use std::sync::{Arc, Mutex};

mod config;
mod gui;
mod serial;
mod utils;

use config::AppConfig;
use gui::HumanoidRobotApp;
use serial::SerialManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Humanoid Robot GUI v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = AppConfig::load().unwrap_or_else(|err| {
        tracing::error!("Failed to load app config, using defaults: {err}");
        AppConfig::default()
    });
    tracing::info!("Configuration loaded");

    // Create serial manager
    let serial_manager = Arc::new(Mutex::new(SerialManager::new(config.serial.clone())));

    // The GUI wrapper is implemented in gui.rs.

    // Create the native options
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Humanoid Robot Control"),
        ..Default::default()
    };

    // Run the application
    let app = HumanoidRobotApp::new(config, serial_manager);
    eframe::run_native(
        "Humanoid Robot Control",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run application: {}", e))?;

    Ok(())
}
