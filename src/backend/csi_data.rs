// --- File: src/backend/csi_data.rs ---
// --- Purpose: Defines the CsiData structure and parsing logic ---

use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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
    pub sig_len_extra: u32,
    pub data_length: u32,
    pub csi_raw_data: Vec<i32>,
}

impl CsiData {
    pub fn parse(input: &str) -> Result<Self, String> {
        let mut data = CsiData::default();
        let mut lines = input.lines();

        while let Some(line) = lines.next() {
            let line = line.trim();
            if line.is_empty() { continue; }

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
                    "noise floor" => data.noise_floor = value.parse().map_err(|_| "Invalid noise floor")?,
                    // ... (Keep other parsers if needed, omitted for brevity but should be there)
                    _ => {}
                }
            }
        }
        Ok(data)
    }

    /// Takes a list of raw packets and produces a single "Averaged" packet
    pub fn average(packets: &[CsiData]) -> Self {
        if packets.is_empty() {
            return CsiData::default();
        }

        let count = packets.len() as i32;
        let first = &packets[0];

        // 1. Prepare sums
        let mut sum_rssi = 0;
        let mut sum_noise = 0;

        // For CSI Data, we assume all packets in this batch have same # of subcarriers
        let subcarrier_len = first.csi_raw_data.len();
        let mut sum_csi = vec![0i64; subcarrier_len];

        for p in packets {
            sum_rssi += p.rssi;
            sum_noise += p.noise_floor;

            for (i, &val) in p.csi_raw_data.iter().enumerate() {
                if i < sum_csi.len() {
                    sum_csi[i] += val as i64;
                }
            }
        }

        // 2. Construct averaged packet
        // We take Metadata (mac, channel) from the most recent packet
        let last = &packets[packets.len() - 1];

        CsiData {
            mac: last.mac.clone(),
            rssi: sum_rssi / count,
            noise_floor: sum_noise / count,
            rate: last.rate,
            channel: last.channel,
            timestamp: last.timestamp, // Use latest timestamp
            sig_len: last.sig_len,
            rx_state: last.rx_state,
            secondary_channel: last.secondary_channel,
            sgi: last.sgi,
            ant: last.ant,
            ampdu_cnt: last.ampdu_cnt,
            sig_mode: last.sig_mode,
            mcs: last.mcs,
            cwb: last.cwb,
            smoothing: last.smoothing,
            not_sounding: last.not_sounding,
            aggregation: last.aggregation,
            stbc: last.stbc,
            fec_coding: last.fec_coding,
            sig_len_extra: last.sig_len_extra,
            data_length: last.data_length,
            csi_raw_data: sum_csi.iter().map(|&x| (x / count as i64) as i32).collect(),
        }
    }
}