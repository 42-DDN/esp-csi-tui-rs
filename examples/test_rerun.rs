// --- File: examples/test_rerun.rs ---
// --- Purpose: Test the Rerun integration with mock CSI data ---

use std::thread;
use std::time::Duration;

// Import from the main project
use project::backend::csi_data::CsiData;
use project::rerun_stream::RerunStreamer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Rerun Integration Test");
    println!("========================\n");

    // Create streamer
    let mut streamer = RerunStreamer::new();

    // Test 1: Start live streaming
    println!("📡 Starting live stream...");
    println!("   (Make sure 'rerun' viewer is running!)");
    
    match streamer.start_stream() {
        Ok(_) => println!("   ✅ Live stream connected successfully"),
        Err(e) => {
            println!("   ⚠️  Live stream failed: {}", e);
            println!("   💡 Start Rerun viewer with: rerun");
        }
    }

    // Test 2: Start RRD recording
    println!("\n💾 Starting RRD recording...");
    match streamer.start_record("logs/test_csi.rrd") {
        Ok(_) => println!("   ✅ RRD recording started: logs/test_csi.rrd"),
        Err(e) => println!("   ❌ RRD recording failed: {}", e),
    }

    // Test 3: Generate and log mock CSI data
    println!("\n📊 Generating mock CSI data...");
    
    for i in 0..20 {
        let mock_csi = generate_mock_csi_packet(i);
        
        streamer.log_csi(&mock_csi);
        
        if i % 5 == 0 {
            println!("   📦 Logged packet {} - RSSI: {}, SNR: {}", 
                i, mock_csi.rssi, mock_csi.rssi - mock_csi.noise_floor);
        }
        
        thread::sleep(Duration::from_millis(100));
    }

    println!("\n✅ Test completed!");
    println!("\n📺 View results:");
    println!("   - Live: Check Rerun viewer window");
    println!("   - Recording: rerun logs/test_csi.rrd");

    // Keep alive for a moment to ensure data is flushed
    thread::sleep(Duration::from_secs(1));

    Ok(())
}

/// Generate a mock CSI packet with realistic data
fn generate_mock_csi_packet(index: u32) -> CsiData {
    use std::f32::consts::PI;
    
    // Generate realistic CSI data (I/Q pairs)
    let mut csi_raw_data = Vec::new();
    let num_subcarriers = 64;
    
    for i in 0..num_subcarriers {
        // Simulate a signal with some variation
        let phase = 2.0 * PI * (i as f32) / (num_subcarriers as f32) + (index as f32) * 0.1;
        let amplitude = 100.0 + 50.0 * (phase * 2.0).sin();
        
        let i_val = (amplitude * phase.cos()) as i32;
        let q_val = (amplitude * phase.sin()) as i32;
        
        csi_raw_data.push(i_val);
        csi_raw_data.push(q_val);
    }

    // Varying RSSI and noise floor
    let rssi = -90 + ((index as f32 * 0.5).sin() * 10.0) as i32;
    let noise_floor = -100 + ((index as f32 * 0.3).cos() * 5.0) as i32;

    CsiData {
        mac: format!("AA:BB:CC:DD:EE:{:02X}", index % 256),
        rssi,
        rate: 54,
        noise_floor,
        channel: 6,
        timestamp: index as u64 * 100,
        sig_len: 1024,
        rx_state: 0,
        secondary_channel: 0,
        sgi: 0,
        ant: 1,
        ampdu_cnt: 0,
        sig_mode: 1,
        mcs: 7,
        cwb: 0,
        smoothing: 1,
        not_sounding: 1,
        aggregation: 0,
        stbc: 0,
        fec_coding: 0,
        sig_len_extra: 0,
        data_length: csi_raw_data.len() as u32,
        csi_raw_data,
    }
}
