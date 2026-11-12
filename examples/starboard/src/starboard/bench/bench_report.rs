use std::time::Duration;

use crate::starboard;

#[derive(Debug, Clone, PartialEq)]
pub struct BenchReport {
    pub config: starboard::bench::ScenarioConfig,
    pub achieved_ops_per_second: f32,
    pub latency: starboard::bench::LatencyPercentiles,
    pub received_bytes_per_second: f64,
    pub dropped_percent: f32,
    pub convergence: Duration,
    pub announcement_bytes: usize,
    pub per_peer_bytes: Vec<starboard::bench::PeerTraffic>,
}

impl BenchReport {
    pub(crate) fn format_duration(duration: Duration) -> String {
        let micros = u64::try_from(duration.as_micros()).unwrap_or(u64::MAX);
        if micros < 1000 {
            return format!("{micros} µs");
        }
        let whole = micros / 1000;
        let tenth = micros.saturating_sub(whole.saturating_mul(1000)) / 100;
        format!("{whole}.{tenth} ms")
    }

    pub(crate) fn format_bytes(bytes: u64) -> String {
        let whole = bytes / 1024;
        let tenth = bytes
            .saturating_sub(whole.saturating_mul(1024))
            .saturating_mul(10)
            / 1024;
        format!("{whole}.{tenth} KiB")
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::BenchReport;

    #[test]
    fn test_usage() {
        assert_eq!(
            BenchReport::format_duration(Duration::from_micros(250)),
            "250 µs"
        );
        assert_eq!(
            BenchReport::format_duration(Duration::from_micros(2_550)),
            "2.5 ms"
        );
        assert_eq!(BenchReport::format_bytes(2_048), "2.0 KiB");
    }
}
