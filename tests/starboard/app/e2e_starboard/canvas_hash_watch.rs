use std::time::Duration;

use bevy::prelude::Resource;

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CanvasHashWatch {
    pub hash: u64,
    pub changed_at: Duration,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::CanvasHashWatch;

    #[test]
    fn test_usage() {
        let watch = CanvasHashWatch::default();
        assert_eq!(watch.hash, 0);
        assert_eq!(watch.changed_at, Duration::ZERO);
    }
}
