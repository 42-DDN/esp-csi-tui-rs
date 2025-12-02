use std::io::{self, BufRead, BufReader};
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::{App, backend};

pub use backend::csi_data;
pub use csi_data::CsiData;

pub fn esp_com(app: Arc<Mutex<App>>) {
    // Switch to mock data for now
    mock_esp_com(app);

    /*
    // Real ESP implementation
    let port_name = "/dev/ttyUSB0";
    let baud_rate = 115200;

    let port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(1000))
        .open();

    match port {
        Ok(port) => {
            let mut reader = BufReader::new(port);

            loop {
                let mut collected_lines = String::new();
                let mut lines_read = 0;
                while lines_read < 24 {
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
                        Err(e) => {}
                    }
                }

                match CsiData::parse(&collected_lines) {
                    Ok(data) => {
                        if let Ok(mut app) = app.lock() {
                            app.dataloader.push_data_packet(data);
                        }
                    }
                    Err(e) => {}
                }
            }
        }
        Err(e) => {}
    }
    */
}

pub fn mock_esp_com(app: Arc<Mutex<App>>) {
    let file_path = "example_data.mock";
    let content = std::fs::read_to_string(file_path).unwrap_or_else(|_| {
        eprintln!("Failed to read mock data file: {}", file_path);
        String::new()
    });

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
        eprintln!("No valid packets found in mock data.");
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

