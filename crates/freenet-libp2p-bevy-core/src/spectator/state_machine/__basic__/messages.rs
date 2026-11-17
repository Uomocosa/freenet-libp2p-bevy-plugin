use bevy::prelude::Message;

use crate::identity;
use crate::p2p;
use crate::spectator;

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum SpectateCommand {
    Start {
        slot: spectator::SpectateSlot,
        zone: identity::ZoneName,
        history_prefix: p2p::HistoryKeyPrefix,
    },
    Stop {
        slot: spectator::SpectateSlot,
    },
}
