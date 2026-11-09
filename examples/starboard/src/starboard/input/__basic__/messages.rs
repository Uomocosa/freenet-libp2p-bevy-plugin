use bevy::prelude::Message;
use derive_more::Deref;

use crate::starboard;

#[derive(Message, Debug, Clone, Copy, Deref)]
pub struct PendingLocalDraw(pub starboard::canvas::DrawOp);
