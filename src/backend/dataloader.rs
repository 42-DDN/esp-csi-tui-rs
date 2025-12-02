// --- File: src/backend/dataloader.rs ---
// --- Purpose: Provides CSI packets (Hardcoded from real samples) ---

use crate::backend::csi_data::CsiData;

/// Returns a specific CSI packet by index.
/// Cycles through 3 real sample packets to simulate a stream.
pub fn get_data_packet(idx: u64) -> Option<CsiData> {

    // Packet 1 Raw Data
    let raw_1 = vec![
        0, 0, 6, 10, 7, 9, 8, 9, 7, 9, 8, 8, 9, 9, 9, 9, 9, 9, 9, 7, 10, 6, 10, 5, 10, 4, 11, 3, 12, 3, 12, 1, 13, 2, 12, 1, 13, 1, 12, 1, 11, 2, 12, 2, 11, 3, 10, 2, 9, 3, 9, 2, 8, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -7, 5, -7, 6, -7, 6, -7, 6, -7, 6, -7, 7, -6, 7, -5, 7, -5, 7, -3, 8, -2, 8, -1, 8, 0, 9, 1, 9, 2, 10, 2, 9, 1, 9, 2, 9, 2, 9, 2, 9, 2, 10, 2, 11, 2, 12, 2, 13, 2, 14, 4, 15
    ];

    // Packet 2 Raw Data
    let raw_2 = vec![
        0, 0, 7, -1, 6, -1, 6, -1, 6, -1, 5, -1, 5, -1, 6, 0, 6, 0, 6, 0, 6, 0, 6, 0, 6, 0, 6, 1, 6, 1, 5, 1, 5, 1, 5, 1, 4, 1, 4, 1, 4, 1, 4, 1, 4, 1, 4, 1, 4, 1, 5, 1, 5, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, -4, 1, -4, 2, -4, 2, -4, 2, -4, 2, -3, 3, -3, 4, -3, 3, -3, 3, -3, 4, -3, 3, -2, 3, -3, 4, -3, 4, -3, 3, -3, 3, -3, 3, -3, 3, -3, 4, -3, 4, -3, 5, -3, 6, -3, 7, -3, 8, -2, 9, -2
    ];

    // Packet 3 Raw Data
    let raw_3 = vec![
        0, 0, 17, 1, 16, 0, 16, 1, 15, 1, 15, 1, 14, 0, 13, 0, 12, -1, 11, -1, 11, -2, 10, -2, 10, -3, 9, -4, 9, -4, 10, -4, 8, -4, 8, -3, 9, -2, 9, -2, 9, -3, 8, -1, 8, -1, 8, -1, 8, -2, 10, -2, 10, -3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 2, 3, 3, 3, 4, 2, 4, 2, 4, 2, 5, 3, 5, 3, 5, 4, 4, 5, 4, 5, 4, 5, 3, 6, 3, 7, 3, 7, 2, 7, 2, 8, 2, 8, 2, 8, 2, 9, 3, 10, 4, 10, 4, 10, 6, 10, 7, 11, 7, 10, 8
    ];

    let packets = vec![
        CsiData {
            mac: "DC:ED:83:4A:55:9A".to_string(),
            rssi: -83,
            rate: 11,
            noise_floor: 161,
            channel: 1,
            timestamp: 3764286,
            sig_len: 28,
            rx_state: 0,
            secondary_channel: 0,
            sgi: 0,
            ant: 0,
            ampdu_cnt: 0,
            sig_mode: 0,
            mcs: 0,
            cwb: 0,
            smoothing: 0,
            not_sounding: 0,
            aggregation: 0,
            stbc: 0,
            fec_coding: 0,
            sig_len_extra: 28,
            data_length: 128,
            csi_raw_data: raw_1,
        },
        CsiData {
            mac: "DC:ED:83:4A:55:9A".to_string(),
            rssi: -84,
            rate: 11,
            noise_floor: 161,
            channel: 1,
            timestamp: 3813428,
            sig_len: 28,
            rx_state: 0,
            secondary_channel: 0,
            sgi: 0,
            ant: 0,
            ampdu_cnt: 0,
            sig_mode: 0,
            mcs: 0,
            cwb: 0,
            smoothing: 0,
            not_sounding: 0,
            aggregation: 0,
            stbc: 0,
            fec_coding: 0,
            sig_len_extra: 28,
            data_length: 128,
            csi_raw_data: raw_2,
        },
        CsiData {
            mac: "DC:ED:83:4A:55:9A".to_string(),
            rssi: -86,
            rate: 11,
            noise_floor: 162,
            channel: 1,
            timestamp: 3859589,
            sig_len: 28,
            rx_state: 0,
            secondary_channel: 0,
            sgi: 0,
            ant: 0,
            ampdu_cnt: 0,
            sig_mode: 0,
            mcs: 0,
            cwb: 0,
            smoothing: 0,
            not_sounding: 0,
            aggregation: 0,
            stbc: 0,
            fec_coding: 0,
            sig_len_extra: 28,
            data_length: 128,
            csi_raw_data: raw_3,
        }
    ];

    let selector = (idx as usize) % packets.len();
    let mut selected_packet = packets[selector].clone();

    // Update timestamp to simulate a continuous stream
    // Assuming 100ms intervals for visual fluidity
    selected_packet.timestamp = idx * 100000;

    Some(selected_packet)
}