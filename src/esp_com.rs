use std::io::{self, BufRead, BufReader};
use std::time::Duration;
use std::sync::{Arc, Mutex};

use crate::{App, backend};

pub use backend::csi_data;
pub use csi_data::CsiData;

pub fn esp_com(app: Arc<Mutex<App>>) {
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
        Err(e) => {}
    }
}
