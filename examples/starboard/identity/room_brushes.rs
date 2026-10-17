use std::collections::{BTreeMap, BTreeSet};

use freenet_libp2p_bevy_plugin::{discovery, identity};

use crate::starboard;

#[must_use]
pub fn room_brushes<'a>(
    members: impl IntoIterator<Item = &'a identity::PeerId>,
    created_at: discovery::UnixTime,
    now: discovery::UnixTime,
) -> BTreeMap<identity::PeerId, starboard::identity::DailyBrush> {
    let sorted_peers: BTreeSet<&identity::PeerId> = members.into_iter().collect();
    let mut taken = BTreeSet::new();
    let mut assigned = BTreeMap::new();

    for peer_id in sorted_peers {
        let base = starboard::identity::daily_brush(peer_id, created_at, now);
        let shape_idx = starboard::identity::SHAPES
            .iter()
            .position(|shape| *shape == base.shape)
            .unwrap_or(0);
        let start_slot = shape_idx
            .saturating_mul(starboard::identity::PALETTE.len())
            .saturating_add(usize::from(*base.color));

        let mut chosen = base;
        for step in 0..starboard::identity::COMBINATION_COUNT {
            let candidate_slot = start_slot
                .wrapping_add(step)
                .checked_rem(starboard::identity::COMBINATION_COUNT)
                .unwrap_or(0);
            let candidate = starboard::identity::brush_at(candidate_slot);
            if !taken.contains(&candidate) {
                chosen = candidate;
                break;
            }
        }
        taken.insert(chosen);
        assigned.insert(peer_id.clone(), chosen);
    }

    assigned
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use freenet_libp2p_bevy_plugin::discovery::UnixTime;
    use freenet_libp2p_bevy_plugin::identity::PeerId;

    use super::room_brushes;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let created_at = UnixTime::from_secs(1_000);
        let now = UnixTime::from_secs(2_000);
        let target = starboard::identity::daily_brush(&PeerId::from("seed-0"), created_at, now);
        let mut colliding = Vec::new();
        for index in 0_u32..50_000 {
            let candidate = PeerId::from(format!("seed-{index}"));
            if starboard::identity::daily_brush(&candidate, created_at, now) == target {
                colliding.push(candidate);
                if colliding.len() == 8 {
                    break;
                }
            }
        }
        assert_eq!(colliding.len(), 8);

        let resolved = room_brushes(&colliding, created_at, now);
        assert_eq!(resolved.len(), 8);
        let unique_values: BTreeSet<_> = resolved.values().copied().collect();
        assert_eq!(unique_values.len(), 8);
    }
}
