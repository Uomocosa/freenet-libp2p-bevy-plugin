use thiserror::Error;

#[derive(Error, Debug)]
pub enum SavedKeypairError {
    #[error("identity file io error")]
    Io(#[from] std::io::Error),
    #[error("keypair codec error")]
    Codec(#[from] libp2p::identity::DecodingError),
}

// no test_usage necessary
