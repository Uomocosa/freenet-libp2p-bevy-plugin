use std::time::Duration;

use derive_more::Display;

use crate::discovery;
use discovery::freenet::basic::constants;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum RequestKind {
    #[display("lobby fetch")]
    FetchLobby,
    #[display("lobby deploy")]
    DeployLobby,
    #[display("peer query")]
    QueryPeers,
    #[display("node status query")]
    QueryNodeStatus,
    #[display("lobby poll")]
    PollLobby,
}

#[rustfmt::skip]
impl RequestKind {
    #[must_use]
    pub const fn timeout(self) -> Duration {
        match self {
            Self::FetchLobby => constants::FETCH_TIMEOUT,
            Self::DeployLobby => constants::DEPLOY_TIMEOUT,
            Self::QueryPeers | Self::QueryNodeStatus => constants::REQUEST_TIMEOUT,
            Self::PollLobby => constants::DRAIN_TIMEOUT,
        }
    }

    pub async fn run_with_timeout<T>(
        self,
        request: impl Future<Output = Result<T, discovery::Error>>,
    ) -> Result<T, discovery::Error> {
        tokio::time::timeout(self.timeout(), request)
            .await
            .unwrap_or(Err(discovery::Error::Timeout(self)))
    }
}

// no test_usage necessary
