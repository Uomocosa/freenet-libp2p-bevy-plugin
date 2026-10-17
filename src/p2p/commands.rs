use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};
use serde::Serialize;

use crate::p2p;

#[derive(Resource, Deref, DerefMut, Serialize)]
pub struct Commands<Payload: p2p::Message>(pub Vec<p2p::Command<Payload>>);

impl<Payload: p2p::Message> Default for Commands<Payload> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

#[rustfmt::skip]
impl<Payload: p2p::Message> Commands<Payload> {
    pub fn take_all(&mut self) -> Vec<p2p::Command<Payload>> {
        self.split_off(0)
    }
}

#[cfg(test)]
mod tests {
    use super::Commands;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let mut c = Commands::<()>::default();
        assert!(c.is_empty());
        c.push(p2p::Command::Net(p2p::NetCommand::Dial(identity::Peer {
            id: identity::PeerId::from("p"),
            addrs: vec![],
        })));
        assert_eq!(c.take_all().len(), 1);
        assert!(c.is_empty());
    }
}
