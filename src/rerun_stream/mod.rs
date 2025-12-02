// --- File: src/rerun_stream/mod.rs ---
// --- Purpose: Rerun.io integration for live streaming and RRD recording of CSI data ---

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use crate::backend::csi_data::CsiData;
use crate::backend::doppler::DopplerSpectrogram;

#[cfg(feature = "rerun")]
use rerun::{RecordingStream, RecordingStreamBuilder};
#[cfg(feature = "rerun")]
use rerun::archetypes::{BarChart, Tensor, Points3D};
#[cfg(feature = "rerun")]
use rerun::components::{Color, Position3D};

// Data Model "CsiFrame"
#[derive(Debug, Clone, Copy)]
pub struct CsiFrame {
    pub timestamp: u64,
    pub subcarriers: [i16; 64],         // raw CSI real/imag pairs (placeholder)
    pub amplitude: [f32; 64],           // parsed
    pub phase: [f32; 64],               // parsed
    pub real: [f32; 64],                // real parts
    pub imag: [f32; 64],                // imaginary parts
}

impl From<&CsiData> for CsiFrame {
    fn from(data: &CsiData) -> Self {
        let mut frame = CsiFrame {
            timestamp: data.timestamp,
            subcarriers: [0; 64],
            amplitude: [0.0; 64],
            phase: [0.0; 64],
            real: [0.0; 64],
            imag: [0.0; 64],
        };

        // Parse raw data (interleaved I/Q)
        for i in 0..64 {
            if 2 * i + 1 < data.csi_raw_data.len() {
                let re = data.csi_raw_data[2 * i] as f32;
                let im = data.csi_raw_data[2 * i + 1] as f32;

                frame.real[i] = re;
                frame.imag[i] = im;
                frame.amplitude[i] = (re * re + im * im).sqrt();
                frame.phase[i] = im.atan2(re);
                frame.subcarriers[i] = re as i16;
            }
        }
        frame
    }
}

pub struct RerunStreamer {
    #[cfg(feature = "rerun")]
    rr: Option<RecordingStream>,
    #[cfg(feature = "rerun")]
    rrd_record: Option<RecordingStream>,
    #[cfg(feature = "rerun")]
    heatmap: VecDeque<[f32; 64]>,
    
    doppler: DopplerSpectrogram,

    app_id: String,
}

impl RerunStreamer {
    pub fn new(app_id: &str) -> Self {
        Self {
            #[cfg(feature = "rerun")]
            rr: None,
            #[cfg(feature = "rerun")]
            rrd_record: None,
            #[cfg(feature = "rerun")]
            heatmap: VecDeque::with_capacity(500),
            
            doppler: DopplerSpectrogram::new(128, 200), // Window=128, History=200

            app_id: app_id.to_string(),
        }
    }

    pub fn connect(&mut self, addr: &str) {
        #[cfg(feature = "rerun")]
        {
            // Handle raw IP/Host addresses by wrapping them in the expected Rerun URL format
            let target = if addr.starts_with("rerun+") {
                addr.to_string()
            } else {
                let host_port = if addr.contains(':') { addr.to_string() } else { format!("{}:9876", addr) };
                format!("rerun+http://{}/proxy", host_port)
            };

            let rec = RecordingStreamBuilder::new(self.app_id.as_str())
                .connect_grpc_opts(target.clone());

            match rec {
                Ok(r) => {self.rr = Some(r);},
                Err(_e) => {}
            }
        }
    }

    pub fn push_csi(&mut self, csi: &CsiFrame) {
        // Update Doppler Spectrogram
        self.doppler.push_frame(csi);

        #[cfg(feature = "rerun")]
        {
            // Update shared heatmap buffer once
            if self.heatmap.len() >= 500 {
                self.heatmap.pop_front();
            }
            self.heatmap.push_back(csi.amplitude);

            // Helper closure to log to a specific stream
            let log_to_stream = |rec: &RecordingStream| {
                rec.set_time_sequence("frame_idx", csi.timestamp as i64);

                // 1. Bar Plot (Amplitude) -> "csi/bar_amplitude"
                let _ = rec.log(
                    "csi/bar_amplitude",
                    &BarChart::new(csi.amplitude.to_vec()),
                );

                // 2. Heatmap -> "csi/heatmap"
                // Convert heatmap buffer to Image (u8 grayscale)
                let height = self.heatmap.len();
                let width = 64;
                let mut img_data = Vec::with_capacity(width * height);

                // Normalize to 0-255
                let max_val = self.heatmap.iter().flatten().fold(0.0f32, |a, &b| a.max(b));
                let scale = if max_val > 0.0 { 255.0 / max_val } else { 0.0 };

                for row in &self.heatmap {
                    for &val in row {
                        img_data.push((val * scale) as u8);
                    }
                }

                let tensor_data = rerun::TensorData::new(
                    vec![height as u64, width as u64],
                    rerun::TensorBuffer::U8(img_data.into())
                );

                let _ = rec.log(
                    "csi/heatmap",
                    &Tensor::new(tensor_data),
                );

                // 3. 3D Scatter -> "csi/complex_scatter"
                let positions: Vec<Position3D> = (0..64).map(|i| {
                    Position3D::new(csi.real[i], csi.imag[i], csi.amplitude[i])
                }).collect();

                let colors: Vec<Color> = (0..64).map(|i| {
                    // Map phase (-PI..PI) to 0..255
                    let p = csi.phase[i];
                    let norm = (p + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
                    let c = (norm * 255.0).clamp(0.0, 255.0) as u8;
                    Color::from_unmultiplied_rgba(c, 100, 255 - c, 255)
                }).collect();

                let _ = rec.log(
                    "csi/complex_scatter",
                    &Points3D::new(positions).with_colors(colors),
                );

                // 4. Doppler Spectrogram -> "csi/doppler_spectrogram"
                self.doppler.to_rerun(rec);
            };

            // Log to Live Stream
            if let Some(rec) = &self.rr {
                log_to_stream(rec);
            }

            // Log to RRD File
            if let Some(rec) = &self.rrd_record {
                log_to_stream(rec);
            }
        }
    }

    pub fn start_record(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(feature = "rerun")]
        {
            // Ensure parent directory exists
            if let Some(parent) = std::path::Path::new(path).parent() {
                std::fs::create_dir_all(parent)?;
            }

            let rec = RecordingStreamBuilder::new(self.app_id.as_str())
                .save(path)?;

            self.rrd_record = Some(rec);
            Ok(())
        }
        #[cfg(not(feature = "rerun"))]
        {
            Err("Rerun feature disabled".into())
        }
    }

    pub fn stop_record(&mut self) {
        #[cfg(feature = "rerun")]
        if let Some(stream) = self.rrd_record.take() {
            drop(stream); // Flushes on drop
        }
    }

    pub fn is_recording(&self) -> bool {
        #[cfg(feature = "rerun")]
        return self.rrd_record.is_some();
        #[cfg(not(feature = "rerun"))]
        false
    }

    pub fn is_connected(&self) -> bool {
        #[cfg(feature = "rerun")]
        return self.rr.is_some();
        #[cfg(not(feature = "rerun"))]
        false
    }

    pub fn disconnect(&mut self) {
        #[cfg(feature = "rerun")]
        {
            self.rr = None;
        }
    }

    pub fn export_history_to_rrd(&self, history: &[CsiData], filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(feature = "rerun")]
        {
            // Ensure parent directory exists
            if let Some(parent) = std::path::Path::new(filename).parent() {
                std::fs::create_dir_all(parent)?;
            }

            let rec = RecordingStreamBuilder::new(self.app_id.as_str())
                .save(filename)?;

            for data in history {
                let frame = CsiFrame::from(data);
                rec.set_time_sequence("frame_idx", frame.timestamp as i64);

                // 1. Bar Plot (Amplitude) -> "csi/bar_amplitude"
                let _ = rec.log(
                    "csi/bar_amplitude",
                    &BarChart::new(frame.amplitude.to_vec()),
                );

                // 2. Heatmap -> "csi/heatmap"
                // (We don't have the heatmap history here, so we skip it or just log the current frame as a row?
                // Actually, the heatmap in push_csi is a rolling buffer.
                // For export, we might just want to log the amplitude as a tensor row if we want a heatmap over time in Rerun.
                // But Rerun handles time series of tensors well.
                // Let's just log the amplitude as a tensor row for now, or skip the heatmap if it's derived.)

                // 3. 3D Scatter -> "csi/complex_scatter"
                let positions: Vec<Position3D> = (0..64).map(|i| {
                    Position3D::new(frame.real[i], frame.imag[i], frame.amplitude[i])
                }).collect();

                let colors: Vec<Color> = (0..64).map(|i| {
                    // Map phase (-PI..PI) to 0..255
                    let p = frame.phase[i];
                    let norm = (p + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
                    let c = (norm * 255.0).clamp(0.0, 255.0) as u8;
                    Color::from_unmultiplied_rgba(c, 100, 255 - c, 255)
                }).collect();

                let _ = rec.log(
                    "csi/complex_scatter",
                    &Points3D::new(positions).with_colors(colors),
                );
            }

            // Explicitly drop rec to flush and close
            drop(rec);
            Ok(())
        }
        #[cfg(not(feature = "rerun"))]
        {
            Err("Rerun feature disabled".into())
        }
    }
}

// Thread-safe wrapper type alias for convenience
pub type SharedRerunStreamer = Arc<Mutex<RerunStreamer>>;

pub fn create_shared_streamer() -> SharedRerunStreamer {
    Arc::new(Mutex::new(RerunStreamer::new("esp-csi-tui")))
}
