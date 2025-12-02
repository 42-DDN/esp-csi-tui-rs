use std::collections::VecDeque;
use rustfft::{FftPlanner, num_complex::Complex};

#[cfg(feature = "rerun")]
use rerun::{RecordingStream, Tensor, TensorData, TensorBuffer};

use crate::rerun_stream::CsiFrame;

pub struct DopplerSpectrogram {
    window_size: usize,
    history_size: usize,
    buffer: VecDeque<f32>, // Sliding window of averaged amplitudes
    spectrogram: VecDeque<Vec<f32>>, // History of FFT frames (Time x Frequency)
    planner: FftPlanner<f32>,
    hann_window: Vec<f32>,
}

impl DopplerSpectrogram {
    pub fn new(window_size: usize, history_size: usize) -> Self {
        // Pre-compute Hann window
        let hann_window: Vec<f32> = (0..window_size)
            .map(|n| 0.5 * (1.0 - (2.0 * std::f32::consts::PI * n as f32 / (window_size as f32 - 1.0)).cos()))
            .collect();

        Self {
            window_size,
            history_size,
            buffer: VecDeque::with_capacity(window_size),
            spectrogram: VecDeque::with_capacity(history_size),
            planner: FftPlanner::new(),
            hann_window,
        }
    }

    pub fn push_frame(&mut self, csi_frame: &CsiFrame) {
        // 1. Preprocessing
        // Compute magnitude for each subcarrier and take the mean
        let mean_amp: f32 = csi_frame.amplitude.iter().sum::<f32>() / csi_frame.amplitude.len() as f32;

        // Append to sliding window buffer
        if self.buffer.len() >= self.window_size {
            self.buffer.pop_front();
        }
        self.buffer.push_back(mean_amp);

        // 2. Sliding Window & FFT
        // Only compute FFT if we have enough samples
        if self.buffer.len() == self.window_size {
            self.generate_fft();
        }
    }

    fn generate_fft(&mut self) {
        let fft = self.planner.plan_fft_forward(self.window_size);
        
        // Prepare input buffer with Hann window applied
        let mut buffer: Vec<Complex<f32>> = self.buffer.iter()
            .zip(self.hann_window.iter())
            .map(|(&val, &win)| Complex::new(val * win, 0.0))
            .collect();

        // 3. Compute FFT
        fft.process(&mut buffer);

        // Compute magnitude |FFT[k]|
        // Since input is real, output is symmetric. We take the first half.
        // But for visualization, keeping full or half depends on preference. 
        // Usually 0 to Nyquist is enough for real signals.
        // Let's keep the first half (0 to N/2).
        let output_len = self.window_size / 2;
        let mut magnitudes: Vec<f32> = buffer.iter()
            .take(output_len)
            .map(|c| c.norm())
            .collect();

        // Normalize magnitudes (simple min-max or just scaling)
        // Let's do a simple log scale or just raw magnitude for now.
        // Task says "Normalize magnitudes".
        let max_val = magnitudes.iter().fold(0.0f32, |a, &b| a.max(b));
        if max_val > 0.0 {
            for x in &mut magnitudes {
                *x /= max_val;
            }
        }

        // 4. Update Spectrogram History
        if self.spectrogram.len() >= self.history_size {
            self.spectrogram.pop_front();
        }
        self.spectrogram.push_back(magnitudes);
    }

    #[cfg(feature = "rerun")]
    pub fn to_rerun(&self, rec: &RecordingStream) {
        if self.spectrogram.is_empty() {
            return;
        }

        // 5. Rerun Visualization
        // X axis = Time (recent FFT frames)
        // Y axis = Frequency bin
        
        let height = self.spectrogram[0].len(); // Frequency bins
        let width = self.spectrogram.len();     // Time history

        // Flatten the 2D deque into a 1D vector for the Tensor
        // We want the image to be [Height, Width] or [Width, Height]
        // Usually Spectrogram is Freq (Y) vs Time (X).
        // So we need to construct the buffer column by column or row by row.
        // Rerun Tensor shape: [H, W] -> Row-major.
        // So we need to iterate rows (frequencies) then columns (time).
        
        let mut img_data = Vec::with_capacity(width * height);

        for freq_idx in 0..height {
            for time_idx in 0..width {
                // Invert Y axis so low freq is at bottom? 
                // Usually index 0 is DC (0Hz). In images, 0 is top.
                // Let's put 0Hz at the bottom (index 0 -> bottom).
                // So we iterate freq from high to low? Or let Rerun handle it.
                // Let's just map 1:1 for now.
                let val = self.spectrogram[time_idx][height - 1 - freq_idx];
                
                // Map 0.0-1.0 to 0-255
                let pixel = (val * 255.0) as u8;
                img_data.push(pixel);
            }
        }

        let tensor_data = TensorData::new(
            vec![height as u64, width as u64],
            TensorBuffer::U8(img_data.into())
        );

        let _ = rec.log(
            "csi/doppler_spectrogram",
            &Tensor::new(tensor_data),
        );
    }
}
