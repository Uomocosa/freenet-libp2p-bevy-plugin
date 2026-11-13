use bevy::prelude::Message;
use derive_more::Deref;

#[derive(Message, Deref, Debug, Clone, PartialEq, Eq)]
pub struct E2EExtensionEffect(pub Vec<u8>);
