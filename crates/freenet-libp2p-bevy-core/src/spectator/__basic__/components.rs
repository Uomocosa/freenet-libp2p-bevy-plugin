use bevy::prelude::Component;

use crate::identity;
use crate::spectator;

#[derive(Component)]
pub struct Spectated<S> {
    pub slot: spectator::SpectateSlot,
    pub zone: identity::ZoneName,
    pub state: S,
    pub freshness: spectator::SpectateFreshness,
}
