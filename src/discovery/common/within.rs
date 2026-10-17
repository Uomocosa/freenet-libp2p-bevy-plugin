use std::time::Duration;

use crate::discovery;

pub async fn within<T>(
    budget: Duration,
    step: discovery::Step,
    request: impl Future<Output = Result<T, discovery::Error>>,
) -> Result<T, discovery::Error> {
    tokio::time::timeout(budget, request)
        .await
        .unwrap_or(Err(discovery::Error::Timeout(step)))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::within;
    use crate::discovery;

    #[tokio::test]
    async fn test_usage() {
        let slow = async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            Ok::<u8, discovery::Error>(1)
        };
        let result = within(Duration::from_millis(5), discovery::Step::Deploy, slow).await;
        assert!(matches!(
            result,
            Err(discovery::Error::Timeout(discovery::Step::Deploy))
        ));

        let fast = async { Ok::<u8, discovery::Error>(7) };
        assert_eq!(
            within(Duration::from_secs(1), discovery::Step::Fetch, fast)
                .await
                .unwrap(),
            7
        );
    }
}
