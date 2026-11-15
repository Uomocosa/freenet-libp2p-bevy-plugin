use std::time::Duration;

use atomic_delegate_macros::atomic_delegates;
use derive_more::Display;

use crate::freenet;
use freenet::{DEPLOY_TIMEOUT, DRAIN_TIMEOUT, FETCH_TIMEOUT, REQUEST_TIMEOUT};

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

impl RequestKind {
    #[must_use]
    pub const fn timeout(self) -> Duration {
        match self {
            Self::FetchLobby => FETCH_TIMEOUT,
            Self::DeployLobby => DEPLOY_TIMEOUT,
            Self::QueryPeers | Self::QueryNodeStatus => REQUEST_TIMEOUT,
            Self::PollLobby => DRAIN_TIMEOUT,
        }
    }
}

#[atomic_delegates]
impl RequestKind {
    pub async fn run_with_timeout<T>(
        self,
        request: impl Future<Output = Result<T, freenet::Error>>,
    ) -> Result<T, freenet::Error> {
    }
}

// no test_usage necessary
