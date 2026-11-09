use thiserror::Error;

#[derive(Error, Debug)]
pub enum PeerBookError {
    #[error("peer book io error")]
    Io(#[from] std::io::Error),
    #[error("peer book codec error")]
    Codec(#[from] serde_json::Error),
}

// no test_usage necessary
