use crate::freenet;

pub async fn run_with_timeout<T>(
    kind: freenet::RequestKind,
    request: impl std::future::Future<Output = Result<T, freenet::Error>>,
) -> Result<T, freenet::Error> {
    tokio::time::timeout(kind.timeout(), request)
        .await
        .unwrap_or(Err(freenet::Error::Timeout(kind)))
}

#[cfg(test)]
mod tests {
    use super::run_with_timeout;
    use crate::freenet;

    #[tokio::test]
    async fn test_usage() {
        let value = run_with_timeout(freenet::RequestKind::QueryPeers, async {
            Ok::<u8, freenet::Error>(7)
        })
        .await;
        assert_eq!(value.ok(), Some(7));
    }
}
