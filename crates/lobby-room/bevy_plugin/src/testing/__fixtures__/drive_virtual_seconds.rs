use std::time::Duration;

pub async fn drive_virtual_seconds(seconds: u64) {
    for _ in 0..seconds.saturating_mul(10) {
        tokio::time::sleep(Duration::from_millis(100)).await;
        tokio::task::yield_now().await;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::drive_virtual_seconds;

    #[tokio::test(start_paused = true)]
    async fn test_usage() {
        let started = tokio::time::Instant::now();
        drive_virtual_seconds(1).await;
        assert!(started.elapsed() >= Duration::from_secs(1));
    }
}
