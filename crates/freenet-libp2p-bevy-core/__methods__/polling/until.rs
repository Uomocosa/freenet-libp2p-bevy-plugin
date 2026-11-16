use std::fmt::Debug;
use std::time::Instant;

use crate::testing;

pub async fn until<T, Probe, Done>(
    polling: testing::harness::Polling,
    label: &str,
    mut probe: Probe,
    done: Done,
) -> Result<T, String>
where
    T: Debug,
    Probe: AsyncFnMut() -> Result<T, String>,
    Done: Fn(&T) -> bool,
{
    let started = Instant::now();
    loop {
        let last = match probe().await {
            Ok(observed) => {
                if done(&observed) {
                    return Ok(observed);
                }
                format!("{observed:?}")
            }
            Err(error) => format!("error: {error}"),
        };
        if started.elapsed() >= *polling {
            return Err(format!(
                "{label}: not reached within {:?}; last observed {last}",
                *polling
            ));
        }
        tokio::time::sleep(testing::harness::POLL_INTERVAL).await;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::until;
    use crate::testing;

    #[tokio::test]
    async fn test_usage() {
        let polling = testing::harness::Polling::from(Duration::from_secs(5));
        let ok = until(polling, "ready", async || Ok(7), |seen| *seen == 7).await;
        assert_eq!(ok, Ok(7));
    }
}
