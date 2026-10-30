use bevy::prelude::Message;
use derive_more::Deref;

use crate::room_sync;

#[derive(Message, Debug, Clone, PartialEq, Eq, Deref)]
pub struct ReplicaInputFromTimer<Op>(pub room_sync::state_machine::Input<Op>);
