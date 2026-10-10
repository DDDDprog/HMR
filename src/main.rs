// Humanoid Robot Desktop Application
// Control interface for Arduino Mega 2560 firmware

mod limits;
mod protocol;

use anyhow::{Context, Result};
use protocol::{Command, RobotSerial};
use std::env;
use std::io::{self, Write};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let port_name = &args[1];
    let command = &args.get(2).map(|s| s.as_str()).unwrap_or("status");

    // Connect to Arduino
    println!("Connecting to {} at 115200 baud...", port_name);
    let mut serial = RobotSerial::open(port_name).context("Failed to open serial port")?;

    println!("Waiting for Arduino to be ready...");
    if !serial.wait_for_ready(5000)? {
        eprintln!("Timeout waiting for Arduino READY signal");
        return Ok(());
    }
    println!("Arduino ready!");

    // Execute command
    match *command {
        "enable" => {
            let resp = serial.send_command(&Command::Enable)?;
            println!("{:?}", resp);
        }
        "disable" => {
            let resp = serial.send_command(&Command::Disable)?;
            println!("{:?}", resp);
        }
        "set" => {
            if args.len() < 5 {
                eprintln!("Usage: {} <port> set <index> <angle>", args[0]);
                return Ok(());
            }
            let index: usize = args[3].parse()?;
            let angle: i32 = args[4].parse()?;

            // Validate angle before sending
            limits::validate_angle(index, angle)
                .map_err(|e| anyhow::anyhow!("Angle validation failed: {}", e))?;

            let resp = serial.send_command(&Command::Set { index, angle })?;
            println!("{:?}", resp);
        }
        "get" => {
            if args.len() < 4 {
                eprintln!("Usage: {} <port> get <index>", args[0]);
                return Ok(());
            }
            let index: usize = args[3].parse()?;
            let resp = serial.send_command(&Command::Get { index })?;
            println!("{:?}", resp);
        }
        "home" => {
            let resp = serial.send_command(&Command::Home)?;
            println!("{:?}", resp);
        }
        "mpu" => {
            let resp = serial.send_command(&Command::Mpu)?;
            println!("{:?}", resp);
        }
        "status" => {
            let resp = serial.send_command(&Command::Status)?;
            println!("{:?}", resp);
        }
        "ping" => {
            let resp = serial.send_command(&Command::Ping)?;
            println!("{:?}", resp);
        }
        "interactive" => {
            run_interactive(&mut serial)?;
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
        }
    }

    Ok(())
}

fn print_usage() {
    println!("Humanoid Robot Control");
    println!();
    println!(
        "Usage: {} <serial_port> <command> [args]",
        env::args().next().unwrap()
    );
    println!();
    println!("Commands:");
    println!("  enable                      Enable servos (attach to pins)");
    println!("  disable                     Disable servos (detach from pins)");
    println!("  set <index> <angle>         Set servo angle (index 0-13, angle depends on servo)");
    println!("  get <index>                 Get current servo angle");
    println!("  home                        Move all servos to home position");
    println!("  mpu                         Read MPU-6050 sensor data");
    println!("  status                      Get robot status");
    println!("  ping                        Ping Arduino");
    println!("  interactive                 Enter interactive mode");
    println!();
    println!("Example:");
    println!("  {} /dev/ttyACM0 enable", env::args().next().unwrap());
    println!("  {} /dev/ttyACM0 set 0 45", env::args().next().unwrap());
    println!("  {} /dev/ttyACM0 interactive", env::args().next().unwrap());
    println!();
    println!("Servo indices:");
    for (i, name) in limits::SERVO_NAMES.iter().enumerate() {
        println!("  {}: {}", i, name);
    }
}

fn run_interactive(serial: &mut RobotSerial) -> Result<()> {
    println!("Interactive mode (type 'help' for commands, 'quit' to exit)");

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "quit" || input == "exit" {
            break;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts.get(0).map(|s| *s).unwrap_or("");

        match cmd {
            "help" => {
                println!("Commands:");
                println!("  enable      - Enable servos");
                println!("  disable     - Disable servos");
                println!("  set <i> <a> - Set servo i to angle a");
                println!("  get <i>     - Get servo i angle");
                println!("  home        - Move all servos to home");
                println!("  mpu         - Read MPU-6050");
                println!("  status      - Get status");
                println!("  ping        - Ping Arduino");
                println!("  servos      - List servo indices and names");
                println!("  quit        - Exit");
            }
            "servos" => {
                for (i, name) in limits::SERVO_NAMES.iter().enumerate() {
                    let limits = limits::SERVO_LIMITS[i];
                    println!(
                        "  {}: {} (min={}, max={}, home={})",
                        i, name, limits.min_angle, limits.max_angle, limits.home_angle
                    );
                }
            }
            "enable" => {
                let resp = serial.send_command(&Command::Enable)?;
                println!("{:?}", resp);
            }
            "disable" => {
                let resp = serial.send_command(&Command::Disable)?;
                println!("{:?}", resp);
            }
            "set" => {
                if parts.len() < 3 {
                    eprintln!("Usage: set <index> <angle>");
                    continue;
                }
                let index: usize = match parts[1].parse() {
                    Ok(i) => i,
                    Err(_) => {
                        eprintln!("Invalid index");
                        continue;
                    }
                };
                let angle: i32 = match parts[2].parse() {
                    Ok(a) => a,
                    Err(_) => {
                        eprintln!("Invalid angle");
                        continue;
                    }
                };

                match limits::validate_angle(index, angle) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        continue;
                    }
                }

                match serial.send_command(&Command::Set { index, angle }) {
                    Ok(resp) => println!("{:?}", resp),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            "get" => {
                if parts.len() < 2 {
                    eprintln!("Usage: get <index>");
                    continue;
                }
                let index: usize = match parts[1].parse() {
                    Ok(i) => i,
                    Err(_) => {
                        eprintln!("Invalid index");
                        continue;
                    }
                };
                match serial.send_command(&Command::Get { index }) {
                    Ok(resp) => println!("{:?}", resp),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            "home" => match serial.send_command(&Command::Home) {
                Ok(resp) => println!("{:?}", resp),
                Err(e) => eprintln!("Error: {}", e),
            },
            "mpu" => match serial.send_command(&Command::Mpu) {
                Ok(resp) => println!("{:?}", resp),
                Err(e) => eprintln!("Error: {}", e),
            },
            "status" => match serial.send_command(&Command::Status) {
                Ok(resp) => println!("{:?}", resp),
                Err(e) => eprintln!("Error: {}", e),
            },
            "ping" => match serial.send_command(&Command::Ping) {
                Ok(resp) => println!("{:?}", resp),
                Err(e) => eprintln!("Error: {}", e),
            },
            _ => {
                eprintln!("Unknown command: {}", cmd);
                eprintln!("Type 'help' for available commands");
            }
        }
    }

    Ok(())
}
