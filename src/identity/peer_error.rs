use thiserror::Error;

#[derive(Error, Debug)]
pub enum PeerError {
    #[error("peer address is empty")]
    Empty,
    #[error("peer address is not a multiaddr")]
    InvalidMultiaddr(#[from] libp2p::multiaddr::Error),
    #[error("peer address is missing a peer id")]
    MissingPeerId,
}

// no test_usage necessary
