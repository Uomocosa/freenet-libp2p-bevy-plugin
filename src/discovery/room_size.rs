use std::num::NonZeroU16;

use bevy::prelude::Reflect;
use derive_more::{Deref, Display};
use serde::{Deserialize, Serialize};

use crate::discovery;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Display,
    Deref,
    Reflect,
)]
pub struct RoomSize(pub NonZeroU16);

impl RoomSize {
    pub const DEFAULT: Self = Self(NonZeroU16::new(8).unwrap());
}

impl TryFrom<u16> for RoomSize {
    type Error = discovery::Error;

    fn try_from(units: u16) -> Result<Self, Self::Error> {
        NonZeroU16::new(units)
            .map(Self)
            .ok_or(discovery::Error::ZeroRoomSize)
    }
}

#[cfg(test)]
mod tests {
    use super::RoomSize;

    #[test]
    fn test_usage() {
        assert!(RoomSize::try_from(0).is_err());
        assert_eq!(RoomSize::try_from(8).ok().map(|size| size.get()), Some(8));
        assert_eq!(RoomSize::DEFAULT.get(), 8);
    }
}
