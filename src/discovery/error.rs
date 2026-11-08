use thiserror::Error;

use crate::freenet;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Freenet(#[from] freenet::Error),
    #[error("room size must be non-zero")]
    ZeroRoomSize,
}

#[rustfmt::skip]
impl From<bincode::Error> for Error {
    fn from(error: bincode::Error) -> Self { Self::Freenet(freenet::Error::from(error)) }
}

// no test_usage necessary
