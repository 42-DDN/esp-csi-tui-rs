You will implement a Doppler Human-Motion Spectrogram based on CSI data in Rust.
Follow these requirements exactly:

FEATURE GOAL:
- Compute a real-time time–frequency spectrogram from CSI magnitude values using a sliding window FFT.
- Visualize this spectrogram in Rerun using rerun bindings.
- The result should highlight human motion (walking, waving, movement) as Doppler shifts.

INPUT:
- CSI frames already parsed from esp-csi-rs.
- Each frame gives N subcarriers, each complex value (real, imag).
- Data arrives one CSI frame at a time in async stream.

PIPELINE:
1. Preprocessing:
   - For each CSI frame, compute magnitude for each subcarrier:
       magnitude[i] = sqrt(re^2 + im^2)
   - Compress across subcarriers by taking the mean magnitude:
       amp = mean(magnitude[i])
   - Append `amp` to a sliding window buffer: VecDeque<f32>

2. Sliding Window:
   - Maintain a fixed-length buffer of size WINDOW_SIZE (e.g., 128 samples)
   - When buffer is full, compute FFT.

3. FFT:
   - Use real-to-complex FFT using the `rustfft` crate.
   - Apply a Hann window before FFT.
   - Compute |FFT[k]| for all k.
   - Normalize magnitudes.

4. Spectrogram Frame:
   - Each FFT output becomes one vertical column in the spectrogram.
   - Store last 200 columns in 2D vec: Vec<Vec<f32>> (time × frequency)

5. Rerun Visualization:
   - Use rerun to send an `Image` or heatmap:
       - X axis = time (recent FFT frames)
       - Y axis = frequency bin (Doppler shift)
       - intensity = amplitude
   - Use rr.log_image() or rr.log_tensor() depending on API version.

STRUCTURE:
- Create module: `doppler`
- Public API:
    pub struct DopplerSpectrogram { ... }
    impl DopplerSpectrogram {
        pub fn new(window_size: usize, history: usize) -> Self
        pub fn push_frame(&mut self, csi_frame: &CSIFrame)
        pub fn generate_fft(&mut self)
        pub fn to_rerun(&self)
    }

- Non-blocking: integrate into existing async loop that processes CSI frames.

ADD COMMENTS explaining:
- buffer management
- FFT preparation
- conversion to rerun tensor

IMPORTANT:
- Do not block the main loop.
- Use efficient data structures.
- Make spectrogram visually appealing.

Finally:
Generate clean, idiomatic Rust code with placeholders where needed.
