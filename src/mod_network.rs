#[derive(Clone, Copy, Debug)]
pub struct NetworkStats {
    pub packet_count: u64,
    pub rssi: i32,
    pub pps: u64,
    pub snr: i32,
    pub timestamp: u64,
}
