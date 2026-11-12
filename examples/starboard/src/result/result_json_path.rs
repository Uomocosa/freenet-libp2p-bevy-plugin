use std::path::PathBuf;
use std::time::Duration;

use bevy::prelude::Resource;
use derive_more::Deref;
use examples_shared::e2e_report::peer_run_result::PeerRunResult;

#[derive(Resource, Deref)]
pub struct ResultJsonPath(pub PathBuf);

#[rustfmt::skip]
impl ResultJsonPath {
    const RESULT_SNAPSHOT_EVERY: Duration = Duration::from_secs(5);

    pub(crate) fn snapshot_every() -> Duration {
        let scale = std::env::var("E2E_TIME_SCALE")
            .ok()
            .and_then(|raw| raw.parse::<f64>().ok())
            .filter(|scale| scale.is_finite() && *scale > 0.0)
            .unwrap_or(1.0);
        if scale >= 1.0 {
            Self::RESULT_SNAPSHOT_EVERY
        } else if scale.to_bits() == 0.2_f64.to_bits() {
            Duration::from_secs(1)
        } else if scale.to_bits() == 0.1_f64.to_bits() {
            Duration::from_millis(500)
        } else {
            Self::RESULT_SNAPSHOT_EVERY
        }
    }

    pub(crate) fn has_room_evidence(result: &PeerRunResult) -> bool {
        !result.members_seen.is_empty() || result.canvas_hash.is_some() || result.error.is_some()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::ResultJsonPath;

    #[test]
    fn test_usage() {
        let path = ResultJsonPath(PathBuf::from("/tmp/result.json"));
        assert_eq!(path.as_path(), PathBuf::from("/tmp/result.json").as_path());
    }
}
