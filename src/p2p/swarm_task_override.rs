use std::marker::PhantomData;

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::Resource;

use crate::identity;
use crate::p2p;
#[cfg(feature = "testing")]
use crate::testing;

#[derive(Resource, Debug, Clone)]
pub struct SwarmTaskOverride<Payload> {
    pub orchestrator_port: u16,
    pub peer: identity::Peer,
    pub marker: PhantomData<Payload>,
}

#[atomic_delegates]
impl<Payload: p2p::Message> SwarmTaskOverride<Payload> {
    pub(crate) fn spawn_fake_swarm_task(
        self,
        commands: tokio::sync::mpsc::UnboundedReceiver<p2p::Command<Payload>>,
        libp2p_commands: tokio::sync::mpsc::UnboundedReceiver<p2p::Libp2pCommand>,
        events: tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
        game: identity::GameId,
    ) -> tokio::task::JoinHandle<()> {
    }
    pub(crate) fn track_room_command(
        &self,
        command: &p2p::Libp2pCommand,
        game: &identity::GameId,
        rooms_by_topic: &mut std::collections::BTreeMap<identity::Topic, identity::RoomName>,
    ) {
    }
    pub(crate) fn handle_server_event(
        &self,
        rooms_by_topic: &std::collections::BTreeMap<identity::Topic, identity::RoomName>,
        event: p2p::Libp2pEvent,
        events: &tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
    ) {
    }
    pub(crate) fn command_frame(
        &self,
        game: &identity::GameId,
        rooms_by_topic: &mut std::collections::BTreeMap<identity::Topic, identity::RoomName>,
        events: &tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
        cmd: p2p::Command<Payload>,
    ) -> Option<testing::FakeWireClientFrame> {
    }
}

#[cfg(test)]
mod tests {
    use super::SwarmTaskOverride;
    use crate::identity;

    #[test]
    fn test_usage() {
        let task_override = SwarmTaskOverride::<u8> {
            orchestrator_port: 4001,
            peer: identity::Peer {
                id: identity::PeerId::from("peer"),
                addrs: Vec::new(),
            },
            marker: std::marker::PhantomData,
        };
        assert_eq!(task_override.orchestrator_port, 4001);
    }
}
