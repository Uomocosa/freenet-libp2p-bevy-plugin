use bevy::prelude::Reflect;
use derive_more::{Deref, Display, From};
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Deref,
    Display,
    From,
    Reflect,
)]
#[reflect(Hash)]
#[from(forward)]
pub struct RoomName(pub String);

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Deref,
    Display,
    From,
    Reflect,
)]
#[reflect(Hash)]
#[from(forward)]
pub struct FreenetPeerId(pub String);

#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Deref,
    Display,
    From,
    Reflect,
)]
#[from(forward)]
pub struct Username(pub String);
