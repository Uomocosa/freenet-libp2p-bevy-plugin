use std::sync::Mutex;

use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};
use tokio::sync::broadcast;

use crate::identity;
use crate::p2p;
use crate::spectator;

#[derive(Resource, Deref)]
pub struct SpectatorGame(pub identity::GameId);

#[derive(Resource, Deref)]
pub struct SpectatorEvents(pub Mutex<broadcast::Receiver<p2p::Libp2pEvent>>);

#[derive(Resource, Deref, DerefMut)]
pub struct SpectatorInputs<Op, Snap>(pub Vec<spectator::state_machine::SpectateInput<Op, Snap>>);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct SpectatorCount(pub usize);

#[derive(Resource)]
pub struct Spectated<S> {
    pub zone: identity::ZoneName,
    pub state: S,
    pub freshness: spectator::SpectateFreshness,
}

// no test_usage necessary
