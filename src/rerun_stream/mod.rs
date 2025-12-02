// --- File: src/rerun_stream/mod.rs ---
// --- Purpose: Rerun.io integration for live streaming and RRD recording of CSI data ---

use std::sync::{Arc, Mutex};
use rerun::{RecordingStream, RecordingStreamBuilder};
use crate::backend::csi_data::CsiData;

/// Rerun streamer managing both live streaming and RRD recording
///
/// # Example Usage
///
/// ```rust,ignore
/// use std::sync::{Arc, Mutex};
/// use rerun_stream::RerunStreamer;
///
/// // Create streamer
/// let streamer = Arc::new(Mutex::new(RerunStreamer::new()));
///
/// // Start live streaming
/// {
///     let mut s = streamer.lock().unwrap();
///     s.start_stream("127.0.0.1:9876").unwrap();
/// }
///
/// // Start RRD recording
/// {
///     let mut s = streamer.lock().unwrap();
///     s.start_record("logs/csi.rrd").unwrap();
/// }
///
/// // Log CSI data
/// {
///     let mut s = streamer.lock().unwrap();
///     if let Some(csi_packet) = get_csi_packet() {
///         s.log_csi(&csi_packet);
///     }
/// }
///
/// // Toggle streaming off
/// {
///     let mut s = streamer.lock().unwrap();
///     s.stop_stream();
/// }
///
/// // Toggle recording off
/// {
///     let mut s = streamer.lock().unwrap();
///     s.stop_record();
/// }
/// ```
pub struct RerunStreamer {
    /// Live streaming connection to Rerun viewer
    live_stream: Option<RecordingStream>,
    /// RRD file recording stream
    rrd_record: Option<RecordingStream>,
}

impl RerunStreamer {
    /// Create a new RerunStreamer with no active connections
    pub fn new() -> Self {
        Self {
            live_stream: None,
            rrd_record: None,
        }
    }

    /// Start live streaming to a Rerun viewer
    ///
    /// Connects to the default Rerun viewer address (127.0.0.1:9876)
    ///
    /// # Returns
    /// * `Ok(())` if connection successful
    /// * `Err(...)` if connection failed
    ///
    /// # Example
    /// ```rust,ignore
    /// streamer.start_stream()?;
    /// ```
    pub fn start_stream(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let rec = RecordingStreamBuilder::new("csi_live")
            .connect_tcp()?;
        
        self.live_stream = Some(rec);
        Ok(())
    }

    /// Stop live streaming
    ///
    /// Gracefully disconnects from the Rerun viewer.
    pub fn stop_stream(&mut self) {
        if let Some(stream) = self.live_stream.take() {
            // RecordingStream automatically flushes on drop
            drop(stream);
        }
    }

    /// Start recording to an RRD file
    ///
    /// # Arguments
    /// * `path` - File path for the .rrd output (e.g., "logs/csi.rrd")
    ///
    /// # Returns
    /// * `Ok(())` if recording started successfully
    /// * `Err(...)` if file creation failed
    ///
    /// # Example
    /// ```rust,ignore
    /// streamer.start_record("logs/csi_session.rrd")?;
    /// ```
    pub fn start_record(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure parent directory exists
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let rec = RecordingStreamBuilder::new("csi_record")
            .save(path)?;
        
        self.rrd_record = Some(rec);
        Ok(())
    }

    /// Stop RRD recording
    ///
    /// Flushes and closes the RRD file.
    pub fn stop_record(&mut self) {
        if let Some(stream) = self.rrd_record.take() {
            // RecordingStream automatically flushes on drop
            drop(stream);
        }
    }

    /// Log a CSI packet to all active endpoints (live stream + RRD)
    ///
    /// Converts CSI data to amplitude arrays and logs to Rerun.
    ///
    /// # Arguments
    /// * `sample` - Reference to the CSI packet to log
    ///
    /// # Example
    /// ```rust,ignore
    /// let csi_packet = CsiData { ... };
    /// streamer.log_csi(&csi_packet);
    /// ```
    pub fn log_csi(&mut self, sample: &CsiData) {
        // Convert CSI raw data to amplitudes
        let amplitudes = self.compute_amplitudes(&sample.csi_raw_data);

        // Log to live stream if active
        if let Some(ref rec) = self.live_stream {
            self.log_to_stream(rec, sample, &amplitudes);
        }

        // Log to RRD recording if active
        if let Some(ref rec) = self.rrd_record {
            self.log_to_stream(rec, sample, &amplitudes);
        }
    }

    /// Check if live streaming is active
    pub fn is_streaming(&self) -> bool {
        self.live_stream.is_some()
    }

    /// Check if RRD recording is active
    pub fn is_recording(&self) -> bool {
        self.rrd_record.is_some()
    }

    // --- Private Helper Methods ---

    /// Compute amplitude array from CSI raw data
    ///
    /// CSI data comes as interleaved I/Q pairs: [I0, Q0, I1, Q1, ...]
    /// Amplitude = sqrt(I^2 + Q^2)
    fn compute_amplitudes(&self, csi_raw: &[i32]) -> Vec<f32> {
        let mut amplitudes = Vec::with_capacity(csi_raw.len() / 2);
        
        for chunk in csi_raw.chunks(2) {
            if chunk.len() == 2 {
                let i = chunk[0] as f32;
                let q = chunk[1] as f32;
                let amplitude = (i * i + q * q).sqrt();
                amplitudes.push(amplitude);
            }
        }
        
        amplitudes
    }

    /// Compute phase array from CSI raw data (optional)
    ///
    /// Phase = atan2(Q, I)
    #[allow(dead_code)]
    fn compute_phases(&self, csi_raw: &[i32]) -> Vec<f32> {
        let mut phases = Vec::with_capacity(csi_raw.len() / 2);
        
        for chunk in csi_raw.chunks(2) {
            if chunk.len() == 2 {
                let i = chunk[0] as f32;
                let q = chunk[1] as f32;
                let phase = q.atan2(i);
                phases.push(phase);
            }
        }
        
        phases
    }

    /// Log CSI data to a specific recording stream
    fn log_to_stream(&self, rec: &RecordingStream, sample: &CsiData, amplitudes: &[f32]) {
        // Log amplitude as a tensor/time series
        let _ = rec.log(
            "csi/amplitude",
            &rerun::Tensor::try_from(amplitudes.to_vec()).unwrap(),
        );

        // Log metadata as scalar values
        let _ = rec.log(
            "csi/rssi",
            &rerun::Scalar::new(sample.rssi as f64),
        );

        let _ = rec.log(
            "csi/noise_floor",
            &rerun::Scalar::new(sample.noise_floor as f64),
        );

        let snr = sample.rssi - sample.noise_floor;
        let _ = rec.log(
            "csi/snr",
            &rerun::Scalar::new(snr as f64),
        );

        let _ = rec.log(
            "csi/channel",
            &rerun::Scalar::new(sample.channel as f64),
        );

        // Log device MAC as text annotation
        let _ = rec.log(
            "csi/device_mac",
            &rerun::TextLog::new(sample.mac.as_str()),
        );
    }
}

impl Default for RerunStreamer {
    fn default() -> Self {
        Self::new()
    }
}

// Thread-safe wrapper type alias for convenience
pub type SharedRerunStreamer = Arc<Mutex<RerunStreamer>>;

/// Create a new shared RerunStreamer instance
///
/// # Example
/// ```rust,ignore
/// let streamer = rerun_stream::create_shared_streamer();
/// ```
pub fn create_shared_streamer() -> SharedRerunStreamer {
    Arc::new(Mutex::new(RerunStreamer::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_amplitudes() {
        let streamer = RerunStreamer::new();
        
        // Test with sample I/Q data
        let csi_raw = vec![3, 4, 5, 12]; // Amplitudes should be 5.0 and 13.0
        let amplitudes = streamer.compute_amplitudes(&csi_raw);
        
        assert_eq!(amplitudes.len(), 2);
        assert!((amplitudes[0] - 5.0).abs() < 0.001);
        assert!((amplitudes[1] - 13.0).abs() < 0.001);
    }

    #[test]
    fn test_new_streamer() {
        let streamer = RerunStreamer::new();
        assert!(!streamer.is_streaming());
        assert!(!streamer.is_recording());
    }

    #[test]
    fn test_stop_before_start() {
        let mut streamer = RerunStreamer::new();
        // Should not panic
        streamer.stop_stream();
        streamer.stop_record();
    }
}
