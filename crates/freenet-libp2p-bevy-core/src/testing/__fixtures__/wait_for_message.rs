use std::time::Duration;

use tokio::sync::mpsc::UnboundedReceiver;

pub async fn wait_for_message<T>(
    messages: &mut UnboundedReceiver<T>,
    timeout: Duration,
    mut matches: impl FnMut(&T) -> bool,
) -> Option<T> {
    tokio::time::timeout(timeout, async {
        while let Some(message) = messages.recv().await {
            if matches(&message) {
                return Some(message);
            }
        }
        None
    })
    .await
    .ok()
    .flatten()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::wait_for_message;

    #[tokio::test]
    async fn test_usage() {
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<u32>();
        sender.send(1).expect("send");
        assert_eq!(
            wait_for_message(&mut receiver, Duration::from_secs(1), |message| *message
                == 1)
            .await,
            Some(1)
        );
        assert_eq!(
            wait_for_message(&mut receiver, Duration::ZERO, |_| true).await,
            None
        );
    }
}
