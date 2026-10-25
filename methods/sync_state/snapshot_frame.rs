use crate::identity;
use crate::p2p;
use crate::room_sync;

pub fn snapshot_frame<Replica: room_sync::RoomState>(
    room: &Replica,
    room_name: identity::RoomName,
    part: u32,
) -> p2p::SnapshotResponse {
    let Ok(bytes) = bincode::serialize(&room.snapshot()) else {
        return p2p::SnapshotResponse::Missing { room: room_name };
    };
    let parts = if bytes.is_empty() {
        vec![Vec::new()]
    } else {
        let width = room_sync::PART_BYTES.max(1);
        bytes.chunks(width).map(<[u8]>::to_vec).collect()
    };
    let total = u32::try_from(parts.len()).unwrap_or(u32::MAX);
    let data = usize::try_from(part)
        .ok()
        .and_then(|index| parts.get(index).cloned());
    match data {
        Some(data) => p2p::SnapshotResponse::Part {
            room: room_name,
            part,
            total,
            data,
        },
        None => p2p::SnapshotResponse::Missing { room: room_name },
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Resource;

    use super::snapshot_frame;
    use crate::identity;
    use crate::p2p;
    use crate::room_sync;

    #[derive(Resource, Default)]
    struct MockReplica;

    impl room_sync::RoomState for MockReplica {
        type Op = u8;
        type Snapshot = Vec<u8>;

        fn apply(&mut self, _meta: room_sync::OpMeta, _op: &u8) {}
        fn snapshot(&self) -> Vec<u8> {
            vec![1, 2, 3]
        }
        fn merge(&mut self, _snapshot: Vec<u8>) {}
        fn slice_content_hashes(&self) -> room_sync::ReplicaSliceContentHashes {
            Vec::new()
        }
        fn slice_snapshot(&self, _index: room_sync::ReplicaSliceIndex) -> Vec<u8> {
            Vec::new()
        }
    }

    #[test]
    fn test_usage() {
        let room = MockReplica;
        let name = identity::RoomName::from("room");
        let frame = snapshot_frame(&room, name.clone(), 0);
        assert!(matches!(
            frame,
            p2p::SnapshotResponse::Part { total: 1, .. }
        ));
        let missing = snapshot_frame(&room, name, 7);
        assert!(matches!(missing, p2p::SnapshotResponse::Missing { .. }));
    }
}
