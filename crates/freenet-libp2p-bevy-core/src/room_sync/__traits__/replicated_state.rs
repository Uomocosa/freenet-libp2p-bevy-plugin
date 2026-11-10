use crate::p2p;
use crate::room_sync;

pub trait ReplicatedState:
    Default + bevy::prelude::Resource<Mutability = bevy::ecs::component::Mutable>
{
    type Op: p2p::traits::Message;
    type Snapshot: p2p::traits::Message;

    fn apply(&mut self, meta: room_sync::OpMeta, op: &Self::Op);
    fn snapshot(&self) -> Self::Snapshot;
    fn merge(&mut self, snapshot: Self::Snapshot);
    fn slice_content_hashes(&self) -> room_sync::ReplicaSliceContentHashes;
    fn slice_snapshot(&self, index: room_sync::ReplicaSliceIndex) -> Self::Snapshot;
}

// no test_usage necessary
