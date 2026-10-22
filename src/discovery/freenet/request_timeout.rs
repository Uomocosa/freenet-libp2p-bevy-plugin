use std::time::Duration;

use derive_more::{Deref, From};

use crate::discovery;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deref, From)]
#[from(forward)]
pub struct RequestTimeout(pub Duration);

#[rustfmt::skip]
impl RequestTimeout {
    pub async fn within<T>(
        &self,
        kind: discovery::FreenetRequestKind,
        request: impl Future<Output = Result<T, discovery::Error>>,
    ) -> Result<T, discovery::Error> {
        tokio::time::timeout(**self, request)
            .await
            .unwrap_or(Err(discovery::Error::Timeout(kind)))
    }
}

// no test_usage necessary
