use freenet_libp2p_bevy_plugin::identity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScenarioConfig {
    pub peers: usize,
    pub ops_per_second: u32,
    pub seconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerTraffic {
    pub peer: identity::PeerId,
    pub received_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RepaintCost {
    pub dirty_tiles: usize,
    pub micros: u64,
}
