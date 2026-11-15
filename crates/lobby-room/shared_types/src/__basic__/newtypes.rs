use derive_more::{Deref, Display, From};
use serde::{Deserialize, Serialize};

use common_types::ZoneName;

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Deref, Display, From,
)]
#[cfg_attr(feature = "bevy", derive(bevy_reflect::Reflect))]
#[cfg_attr(feature = "bevy", reflect(Hash))]
#[from(forward)]
pub struct RoomName(pub ZoneName);
