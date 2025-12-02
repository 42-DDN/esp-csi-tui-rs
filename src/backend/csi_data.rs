// --- File: src/backend/csi_data.rs ---
// --- Purpose: Defines the CsiData structure and parsing logic ---

#[derive(Debug, Default, Clone)]
pub struct CsiData {
    pub mac: String,
    pub rssi: i32,
    pub rate: u32,
    pub noise_floor: i32,
    pub channel: u32,
    pub timestamp: u64,
    pub sig_len: u32,
    pub rx_state: u32,
    pub secondary_channel: u32,
    pub sgi: u32,
    pub ant: u32,
    pub ampdu_cnt: u32,
    pub sig_mode: u32,
    pub mcs: u32,
    pub cwb: u32,
    pub smoothing: u32,
    pub not_sounding: u32,
    pub aggregation: u32,
    pub stbc: u32,
    pub fec_coding: u32,
    pub sig_len_extra: u32, // Corresponds to the second "sig_len" in the output
    pub data_length: u32,
    pub csi_raw_data: Vec<i32>,
}

impl CsiData {
    pub fn parse(input: &str) -> Result<Self, String> {
        let mut data = CsiData::default();
        let mut lines = input.lines();

        while let Some(line) = lines.next() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line == "csi raw data:" {
                if let Some(data_line) = lines.next() {
                    let content = data_line.trim().trim_start_matches('[').trim_end_matches(']');
                    if !content.is_empty() {
                        data.csi_raw_data = content
                            .split(',')
                            .map(|s| s.trim().parse::<i32>())
                            .collect::<Result<Vec<_>, _>>()
                            .map_err(|e| format!("Failed to parse csi data: {}", e))?;
                    }
                }
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "mac" => data.mac = value.to_string(),
                    "rssi" => data.rssi = value.parse().map_err(|_| "Invalid rssi")?,
                    "rate" => data.rate = value.parse().map_err(|_| "Invalid rate")?,
                    "noise floor" => data.noise_floor = value.parse().map_err(|_| "Invalid noise floor")?,
                    "channel" => data.channel = value.parse().map_err(|_| "Invalid channel")?,
                    "timestamp" => data.timestamp = value.parse().map_err(|_| "Invalid timestamp")?,
                    "sig len" => data.sig_len = value.parse().map_err(|_| "Invalid sig len")?,
                    "rx state" => data.rx_state = value.parse().map_err(|_| "Invalid rx state")?,
                    "secondary channel" => {
                        data.secondary_channel = value.parse().map_err(|_| "Invalid secondary channel")?
                    }
                    "sgi" => data.sgi = value.parse().map_err(|_| "Invalid sgi")?,
                    "ant" => data.ant = value.parse().map_err(|_| "Invalid ant")?,
                    "ampdu cnt" => data.ampdu_cnt = value.parse().map_err(|_| "Invalid ampdu cnt")?,
                    "sig_mode" => data.sig_mode = value.parse().map_err(|_| "Invalid sig_mode")?,
                    "mcs" => data.mcs = value.parse().map_err(|_| "Invalid mcs")?,
                    "cwb" => data.cwb = value.parse().map_err(|_| "Invalid cwb")?,
                    "smoothing" => data.smoothing = value.parse().map_err(|_| "Invalid smoothing")?,
                    "not sounding" => data.not_sounding = value.parse().map_err(|_| "Invalid not sounding")?,
                    "aggregation" => data.aggregation = value.parse().map_err(|_| "Invalid aggregation")?,
                    "stbc" => data.stbc = value.parse().map_err(|_| "Invalid stbc")?,
                    "fec coding" => data.fec_coding = value.parse().map_err(|_| "Invalid fec coding")?,
                    "sig_len" => data.sig_len_extra = value.parse().map_err(|_| "Invalid sig_len_extra")?,
                    "data length" => data.data_length = value.parse().map_err(|_| "Invalid data length")?,
                    _ => {} // Ignore unknown fields
                }
            }
        }
        Ok(data)
    }
}