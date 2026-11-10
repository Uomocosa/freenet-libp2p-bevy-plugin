use derive_more::{Deref, Display, From};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Deref, Display, From,
)]
#[cfg_attr(feature = "bevy", derive(bevy_reflect::Reflect))]
#[cfg_attr(feature = "bevy", reflect(Hash))]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
#[from(forward)]
pub struct RoomName(pub String);

#[derive(
    Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Deref, Display, From,
)]
#[cfg_attr(feature = "bevy", derive(bevy_reflect::Reflect))]
#[from(forward)]
pub struct Username(pub String);
