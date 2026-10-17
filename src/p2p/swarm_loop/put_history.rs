use libp2p::kad;

use crate::p2p;

pub fn put_history<T: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<T>>,
    id: &p2p::HistoryChunkId,
    data: Vec<u8>,
) {
    let record_key = p2p::record_key(id);
    let record = kad::Record {
        key: record_key.clone(),
        value: data,
        publisher: None,
        expires: None,
    };
    let _ = swarm
        .behaviour_mut()
        .kademlia
        .put_record(record, kad::Quorum::One);
    let _ = swarm.behaviour_mut().kademlia.start_providing(record_key);
}

// no test_usage necessary
