use bevy::prelude::Message;
use derive_more::Deref;

use crate::identity;
use crate::replica_sync;

#[derive(Message, Debug, Clone, PartialEq, Eq, Deref)]
pub struct ReplicaInputFromTimer<Op>(pub replica_sync::state_machine::Input<Op>);

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum JoinFailure {
    RoomGone(identity::RoomName),
    HistoryUnavailable(identity::RoomName),
}
