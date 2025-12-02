use std::io::{self, BufRead, BufReader};
use std::time::Duration;

mod csi_data;
use csi_data::CsiData;

fn esp_com() {
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
                        Err(e) => {
                            eprintln!("Error reading serial port: {:?}", e);
                            break;
                        }
                    }
                }

                match CsiData::parse(&collected_lines) {
                    Ok(data) => println!("Parsed Data: {:#?}", data),
                    Err(e) => eprintln!("Parse Error: {}", e),
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            ::std::process::exit(1);
        }
    }
}
