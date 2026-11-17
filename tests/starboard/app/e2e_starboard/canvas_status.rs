use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanvasStatus {
    pub hash: u64,
    pub painted_tiles: usize,
    pub stable_for_ms: u64,
    pub spectators: usize,
}

#[cfg(test)]
mod tests {
    use super::CanvasStatus;

    #[test]
    fn test_usage() {
        let status = CanvasStatus {
            hash: 7,
            painted_tiles: 2,
            stable_for_ms: 300,
            spectators: 1,
        };
        let json = serde_json::to_value(status).expect("serialize");
        let back: CanvasStatus = serde_json::from_value(json).expect("deserialize");
        assert_eq!(back, status);
    }
}
