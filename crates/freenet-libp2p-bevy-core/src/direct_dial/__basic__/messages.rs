use bevy::prelude::Message;
use derive_more::Deref;

#[derive(Message, Debug, Clone, PartialEq, Eq, Deref)]
pub struct FriendAddressSubmitted(pub String);
