use std::io::{self, Read, Write};
use std::thread;
use std::time::Duration;
use serialport::SerialPort;

/// Resets the ESP device and sends the "start" command.
pub fn reset_and_start_esp(port: &mut Box<dyn SerialPort>) -> io::Result<()> {
    // 1. Reset the device using DTR/RTS (Standard ESP32/ESP8266 logic)
    // RTS=True pulls EN low (Reset)
    port.write_data_terminal_ready(false)?;
    port.write_request_to_send(true)?;
    thread::sleep(Duration::from_millis(100));

    // Release Reset
    port.write_request_to_send(false)?;
    
    // 2. Wait for boot and prompt
    // The ESP sends a welcome message ending with "> " when ready.
    let start = std::time::Instant::now();
    let mut buf = [0u8; 1];
    while start.elapsed() < Duration::from_secs(10) {
        match port.read(&mut buf) {
            Ok(n) if n > 0 => {
                if buf[0] == b'>' {
                    break;
                }
            }
            Ok(_) => continue,
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => continue,
            Err(e) => {
                return Err(e);
            }
        }
    }

    // 3. Send "start" command
    port.write_all(b"start\r\n")?;
    port.flush()?;

    Ok(())
}
