use std::marker::PhantomData;

use crate::identity;
use crate::p2p;

#[must_use]
pub const fn swarm_task_override<Payload: p2p::traits::Message>(
    orchestrator_port: u16,
    peer: identity::Peer,
) -> p2p::SwarmTaskOverride<Payload> {
    p2p::SwarmTaskOverride {
        orchestrator_port,
        peer,
        marker: PhantomData,
    }
}

#[cfg(test)]
mod tests {
    use crate::testing;

    use super::swarm_task_override;

    #[test]
    fn test_usage() {
        let task = swarm_task_override::<u8>(1, testing::peer("peer", &[]));
        assert_eq!(task.orchestrator_port, 1);
        assert_eq!(task.peer.id.as_str(), "peer");
    }
}
