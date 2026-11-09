use thiserror::Error;

#[derive(Error, Debug)]
pub enum RoomSizeError {
    #[error("room size must be non-zero")]
    ZeroRoomSize,
}

// no test_usage necessary
