use std::time::Duration;

use freenet_libp2p_bevy_plugin::identity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScenarioConfig {
    pub peers: usize,
    pub ops_per_second: u32,
    pub seconds: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LatencyPercentiles {
    pub p50: Duration,
    pub p99: Duration,
    pub samples: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerTraffic {
    pub peer: identity::PeerId,
    pub received_bytes: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BenchReport {
    pub config: ScenarioConfig,
    pub achieved_ops_per_second: f32,
    pub latency: LatencyPercentiles,
    pub received_bytes_per_second: f64,
    pub dropped_percent: f32,
    pub convergence: Duration,
    pub announcement_bytes: usize,
    pub per_peer_bytes: Vec<PeerTraffic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RepaintCost {
    pub dirty_tiles: usize,
    pub micros: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MachineInfo {
    pub kernel: String,
    pub cpu: String,
    pub cores: usize,
}
