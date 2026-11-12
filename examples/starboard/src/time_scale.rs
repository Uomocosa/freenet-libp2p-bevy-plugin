use std::time::Duration;

use derive_more::Deref;

#[derive(Debug, Clone, Copy, PartialEq, Deref)]
pub struct Scale(pub f64);

impl Scale {
    pub(crate) fn from_env() -> Self {
        Self(
            std::env::var("E2E_TIME_SCALE")
                .ok()
                .and_then(|raw| raw.parse::<f64>().ok())
                .filter(|scale| scale.is_finite() && *scale > 0.0)
                .unwrap_or(1.0),
        )
    }

    pub(crate) fn scaled(self, duration: Duration) -> Duration {
        let seconds = duration.as_secs_f64() * self.max(0.0);
        if seconds <= 0.0 {
            return Duration::ZERO;
        }
        Duration::from_secs_f64(seconds)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Scale;

    #[test]
    fn test_usage() {
        assert_eq!(
            Scale(2.0).scaled(Duration::from_secs(3)),
            Duration::from_secs(6)
        );
        assert_eq!(Scale(0.0).scaled(Duration::from_secs(3)), Duration::ZERO);
    }
}
