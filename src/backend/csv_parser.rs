use std::error::Error;
use std::fs::File;
use serde::Deserialize;
use super::csi_data::CsiData;

#[derive(Debug, Deserialize)]
struct CsiDataCsvRow {
    mac: String,
    rssi: i32,
    rate: u32,
    noise_floor: i32,
    channel: u32,
    timestamp: u64,
    sig_len: u32,
    rx_state: u32,
    secondary_channel: u32,
    sgi: u32,
    ant: u32,
    ampdu_cnt: u32,
    sig_mode: u32,
    mcs: u32,
    cwb: u32,
    smoothing: u32,
    not_sounding: u32,
    aggregation: u32,
    stbc: u32,
    fec_coding: u32,
    sig_len_extra: u32,
    data_length: u32,
    csi_raw_data: String,
}

pub struct CsvParser;

impl CsvParser {
    pub fn parse_csv(path: &str) -> Result<Vec<CsiData>, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut data_list = Vec::new();

        for result in rdr.deserialize() {
            let record: CsiDataCsvRow = result?;
            
            // Parse the "[1, 2, 3]" string back into Vec<i32>
            let raw_str = record.csi_raw_data.trim();
            let raw_str = raw_str.trim_matches(|c| c == '[' || c == ']');
            
            let csi_raw_data: Vec<i32> = if raw_str.is_empty() {
                Vec::new()
            } else {
                raw_str.split(',')
                    .map(|s| s.trim().parse::<i32>())
                    .collect::<Result<Vec<i32>, _>>()?
            };

            let csi_data = CsiData {
                mac: record.mac,
                rssi: record.rssi,
                rate: record.rate,
                noise_floor: record.noise_floor,
                channel: record.channel,
                timestamp: record.timestamp,
                sig_len: record.sig_len,
                rx_state: record.rx_state,
                secondary_channel: record.secondary_channel,
                sgi: record.sgi,
                ant: record.ant,
                ampdu_cnt: record.ampdu_cnt,
                sig_mode: record.sig_mode,
                mcs: record.mcs,
                cwb: record.cwb,
                smoothing: record.smoothing,
                not_sounding: record.not_sounding,
                aggregation: record.aggregation,
                stbc: record.stbc,
                fec_coding: record.fec_coding,
                sig_len_extra: record.sig_len_extra,
                data_length: record.data_length,
                csi_raw_data,
            };
            
            data_list.push(csi_data);
        }

        Ok(data_list)
    }
}
