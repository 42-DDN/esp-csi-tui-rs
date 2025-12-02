use std::io::{self, BufRead, BufReader};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::{App, backend};
use crate::app::DataSource;

pub use backend::csi_data;
pub use csi_data::CsiData;

pub fn esp_com(app: Arc<Mutex<App>>) {
    loop {
        let source = {
            let mut guard = app.lock().unwrap();
            if guard.should_quit {
                break;
            }
            guard.should_switch_source = false;
            guard.data_source.clone()
        };

        match source {
            DataSource::Serial => run_serial(Arc::clone(&app)),
            DataSource::FileReplay(path) => run_replay(Arc::clone(&app), path),
        }
        
        thread::sleep(Duration::from_millis(100));
    }
}

fn run_serial(app: Arc<Mutex<App>>) {
    let ports = serialport::available_ports().unwrap_or_default();
    let port_name = ports
        .iter()
        .find(|p| matches!(p.port_type, serialport::SerialPortType::UsbPort(_)))
        .map(|p| p.port_name.clone())
        .unwrap_or_else(|| "/dev/ttyUSB0".to_string());

    let baud_rate = 115200;
    let port = serialport::new(&port_name, baud_rate)
        .timeout(Duration::from_millis(1000))
        .open();

    match port {
        Ok(mut port) => {
            let mut reader = BufReader::new(port.try_clone().expect("Failed to clone port"));

            loop {
                // Check for exit/switch conditions
                if let Ok(guard) = app.lock() {
                    if guard.should_quit || guard.should_switch_source {
                        break;
                    }
                }

                // Check for Reset Command
                let should_reset = if let Ok(app) = app.lock() {
                    app.should_reset_esp
                } else {
                    false
                };

                if should_reset {
                    if let Err(_e) = backend::esp_utility::reset_and_start_esp(&mut port) {}
                    if let Ok(mut app) = app.lock() {
                        app.should_reset_esp = false;
                    }
                }

                let mut collected_lines = String::new();
                let mut lines_read = 0;
                while lines_read < 24 {
                    if let Ok(guard) = app.lock() {
                        if guard.should_reset_esp || guard.should_quit || guard.should_switch_source {
                            break;
                        }
                    }

                    let mut line = String::new();
                    match reader.read_line(&mut line) {
                        Ok(len) => {
                            if len > 0 {
                                collected_lines.push_str(&line);
                                lines_read += 1;
                            }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                            // Check conditions again on timeout
                            if let Ok(guard) = app.lock() {
                                if guard.should_quit || guard.should_switch_source {
                                    break;
                                }
                            }
                            continue;
                        }
                        Err(_e) => {}
                    }
                }

                if let Ok(data) = CsiData::parse(&collected_lines) {
                    push_data_to_app(&app, data);
                }
            }
        }
        Err(_) => {
            // If serial fails, sleep a bit and return to main loop (which might retry)
            thread::sleep(Duration::from_secs(1));
        }
    }
}

fn run_replay(app: Arc<Mutex<App>>, path: String) {
    // Load CSV
    let packets = match backend::csv_parser::CsvParser::parse_csv(&path) {
        Ok(p) => p,
        Err(_e) => {
            // Log error or just return
            return;
        }
    };

    if packets.is_empty() {
        return;
    }

    let mut index = 0;
    loop {
        if let Ok(guard) = app.lock() {
            if guard.should_quit || guard.should_switch_source {
                break;
            }
        }

        if index < packets.len() {
            let mut packet = packets[index].clone();
            // Update timestamp to simulate live data
            packet.timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64;

            push_data_to_app(&app, packet);

            index += 1;
            thread::sleep(Duration::from_millis(10));
        } else {
            thread::sleep(Duration::from_millis(100));
        }
    }
}

fn push_data_to_app(app: &Arc<Mutex<App>>, data: CsiData) {
    if let Ok(mut app) = app.lock() {
        app.dataloader.push_data_packet(data.clone());

        // Log to Rerun if enabled
        if let Some(ref streamer) = app.rerun_streamer {
            if let Ok(mut s) = streamer.lock() {
                #[cfg(feature = "rerun")]
                {
                    let frame = crate::rerun_stream::CsiFrame::from(&data);
                    s.push_csi(&frame);
                }
            }
        }
    }
}
