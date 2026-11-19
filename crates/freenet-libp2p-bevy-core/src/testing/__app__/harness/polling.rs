use std::fmt::Debug;
use std::time::Duration;

use atomic_delegate_macros::atomic_delegates;
use derive_more::Deref;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deref)]
pub struct Polling(pub Duration);

impl From<Duration> for Polling {
    fn from(deadline: Duration) -> Self {
        Self(deadline)
    }
}

#[atomic_delegates]
impl Polling {
    pub async fn until<T, Probe, Done>(
        self,
        label: &str,
        probe: Probe,
        done: Done,
    ) -> Result<T, String>
    where
        T: Debug,
        Probe: AsyncFnMut() -> Result<T, String>,
        Done: Fn(&T) -> bool,
    {
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Polling;

    #[tokio::test]
    async fn test_usage() {
        let ok = Polling::from(Duration::from_secs(5))
            .until("ready", async || Ok(7), |seen| *seen == 7)
            .await;
        assert_eq!(ok, Ok(7));
        let slow = Polling::from(Duration::from_millis(0))
            .until("never", async || Ok(0), |seen| *seen == 1)
            .await;
        assert!(slow.is_err());
    }
}
