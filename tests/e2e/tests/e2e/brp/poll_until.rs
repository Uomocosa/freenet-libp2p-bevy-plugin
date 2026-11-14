use std::fmt::Debug;
use std::time::{Duration, Instant};

pub const POLL_INTERVAL: Duration = Duration::from_millis(250);

/// Polls `probe` every [`POLL_INTERVAL`] until `done` accepts a value.
///
/// # Errors
/// Returns an error naming `label`, the last observed value (or error), once `deadline` passes.
pub async fn poll_until<T, Probe, Done>(
    label: &str,
    deadline: Duration,
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
        if started.elapsed() >= deadline {
            return Err(format!(
                "{label}: not reached within {deadline:?}; last observed {last}"
            ));
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}
