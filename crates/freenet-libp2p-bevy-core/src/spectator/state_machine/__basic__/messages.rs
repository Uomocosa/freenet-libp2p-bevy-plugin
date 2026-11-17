use bevy::prelude::Message;

use crate::identity;
use crate::p2p;

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum SpectateCommand {
    Start {
        zone: identity::ZoneName,
        history_prefix: p2p::HistoryKeyPrefix,
    },
    Stop,
}
