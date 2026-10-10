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
pub struct PeerId(pub String);

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
pub struct PeerAddr(pub String);

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
pub struct Topic(pub String);
