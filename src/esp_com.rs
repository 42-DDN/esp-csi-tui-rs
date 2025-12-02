use std::io::{self, BufRead, BufReader};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::{App, backend};

pub use backend::csi_data;
pub use csi_data::CsiData;

pub fn esp_com(app: Arc<Mutex<App>>) {
    // Switch to mock data for now
    // mock_esp_com(app);

    // Real ESP implementation
    let ports = serialport::available_ports().unwrap_or_default();

    // Find first USB port, or fallback to default /dev/ttyUSB0
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
                    // Re-create reader after reset might be needed if the port state changes significantly,
                    // but usually just flushing is enough.
                    // However, reset_and_start_esp writes to the port.
                }

                let mut collected_lines = String::new();
                let mut lines_read = 0;
                while lines_read < 24 {
                    // Check for reset request
                    if let Ok(guard) = app.lock() {
                        if guard.should_reset_esp {
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
                            continue;
                        }
                        Err(_e) => {}
                    }
                }

                match CsiData::parse(&collected_lines) {
                    Ok(data) => {
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
                    Err(_e) => {}
                }
            }
        }
        Err(_e) => {}
    }
}

pub fn mock_esp_com(app: Arc<Mutex<App>>) {
    let file_path = "example_data.mock";
    let content = std::fs::read_to_string(file_path).unwrap_or_else(|_| String::new());

    let mut packets = Vec::new();
    let mut current_chunk = String::new();
    let mut line_count = 0;

    for line in content.lines() {
        current_chunk.push_str(line);
        current_chunk.push('\n');
        line_count += 1;

        if line_count == 24 {
            if let Ok(data) = CsiData::parse(&current_chunk) {
                packets.push(data);
            }
            current_chunk.clear();
            line_count = 0;
        }
    }

    if packets.is_empty() {
        return;
    }

    let mut index = 0;
    loop {
        let mut packet = packets[index].clone();

        // Update timestamp to simulate live data
        // Using microsecond timestamp similar to ESP
        packet.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;

        if let Ok(mut app_guard) = app.lock() {
            app_guard.dataloader.push_data_packet(packet);
        }

        index = (index + 1) % packets.len();
        thread::sleep(Duration::from_millis(100)); // 10Hz
    }
}
