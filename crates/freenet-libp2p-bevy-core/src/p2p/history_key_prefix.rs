use bevy::prelude::Component;
use derive_more::{Deref, From};
use serde::{Deserialize, Serialize};

use crate::identity;
use crate::time;

#[derive(
    Component,
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
    From,
)]
#[from(forward)]
pub struct HistoryKeyPrefix(pub String);

impl HistoryKeyPrefix {
    #[must_use]
    pub fn new(
        game: &identity::GameId,
        zone: &identity::ZoneName,
        created_at: time::UnixTime,
    ) -> Self {
        let digest = blake3::hash(
            format!(
                "{}/{}/{}",
                game.as_str(),
                zone.as_str(),
                created_at.as_secs()
            )
            .as_bytes(),
        );
        Self::from(digest.to_hex().to_string())
    }
}
