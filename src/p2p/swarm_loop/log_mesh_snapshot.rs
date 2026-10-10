use crate::p2p;

pub fn log_mesh_snapshot<T: p2p::Message>(swarm: &libp2p::Swarm<p2p::Behaviour<T>>) {
    let total = swarm.behaviour().gossipsub.all_mesh_peers().count();
    let mut topics = Vec::new();
    for topic in swarm.behaviour().gossipsub.topics() {
        let depth = swarm.behaviour().gossipsub.mesh_peers(topic).count();
        topics.push(format!("{topic}={depth}"));
    }
    tracing::debug!(target: "p2p", total, topics = ?topics, "p2p mesh snapshot");
}

// no test_usage necessary
