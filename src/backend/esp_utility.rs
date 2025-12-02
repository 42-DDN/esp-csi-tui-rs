use std::io::{self, Write};
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
    
    // 2. Wait for boot (approx 500ms usually sufficient for bootloader to hand over)
    thread::sleep(Duration::from_millis(500));

    // 3. Send "start" command
    port.write_all(b"start\n")?;
    port.flush()?;

    Ok(())
}
