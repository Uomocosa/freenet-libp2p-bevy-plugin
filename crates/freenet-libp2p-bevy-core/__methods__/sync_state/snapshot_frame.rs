use crate::identity;
use crate::p2p;
use crate::replica_sync;

pub fn snapshot_frame<Replica: replica_sync::traits::ReplicatedState>(
    zone: &Replica,
    zone_name: identity::ZoneName,
    part: u32,
) -> p2p::SnapshotResponse {
    let Ok(bytes) = bincode::serialize(&zone.snapshot()) else {
        return p2p::SnapshotResponse::Missing { zone: zone_name };
    };
    let parts = if bytes.is_empty() {
        vec![Vec::new()]
    } else {
        let width = replica_sync::PART_BYTES.max(1);
        bytes.chunks(width).map(<[u8]>::to_vec).collect()
    };
    let total = u32::try_from(parts.len()).unwrap_or(u32::MAX);
    let data = usize::try_from(part)
        .ok()
        .and_then(|index| parts.get(index).cloned());
    match data {
        Some(data) => p2p::SnapshotResponse::Part {
            zone: zone_name,
            part,
            total,
            data,
        },
        None => p2p::SnapshotResponse::Missing { zone: zone_name },
    }
}

#[cfg(test)]
mod tests {
    use super::snapshot_frame;
    use crate::identity;
    use crate::p2p;
    use crate::testing;

    #[test]
    fn test_usage() {
        let zone = testing::FakeReplica::<u8, Vec<u8>> {
            stored_snapshot: vec![1, 2, 3],
            ..Default::default()
        };
        let name = identity::ZoneName::from("zone");
        let frame = snapshot_frame(&zone, name.clone(), 0);
        assert!(matches!(
            frame,
            p2p::SnapshotResponse::Part { total: 1, .. }
        ));
        let missing = snapshot_frame(&zone, name, 7);
        assert!(matches!(missing, p2p::SnapshotResponse::Missing { .. }));
    }
}
