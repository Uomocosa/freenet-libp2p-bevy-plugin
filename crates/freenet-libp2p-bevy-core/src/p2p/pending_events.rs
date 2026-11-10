use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};
use serde::Serialize;

use crate::p2p;

#[derive(Resource, Deref, DerefMut, Serialize)]
pub struct PendingEvents<Payload: p2p::traits::Message>(pub Vec<p2p::Event<Payload>>);

impl<Payload: p2p::traits::Message> Default for PendingEvents<Payload> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

#[rustfmt::skip]
impl<Payload: p2p::traits::Message> PendingEvents<Payload> {
    pub fn take_all(&mut self) -> Vec<p2p::Event<Payload>> {
        self.split_off(0)
    }
}

#[cfg(test)]
mod tests {
    use super::PendingEvents;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let mut e = PendingEvents::<()>::default();
        assert!(e.is_empty());
        e.push(p2p::Event::Error("oops".to_string()));
        assert_eq!(e.take_all().len(), 1);
        assert!(e.is_empty());
    }
}
