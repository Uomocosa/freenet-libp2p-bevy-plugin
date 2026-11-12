use bevy::prelude::Image;
use freenet_libp2p_bevy_plugin::identity;
use freenet_libp2p_bevy_plugin::time;
use serde::{Deserialize, Serialize};

use crate::starboard;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DailyBrush {
    pub shape: starboard::identity::Shape,
    pub color: starboard::identity::PaletteColor,
}

impl DailyBrush {
    #[must_use]
    pub fn from_peer(
        peer_id: &identity::PeerId,
        created_at: time::UnixTime,
        now: time::UnixTime,
    ) -> Self {
        let day = starboard::identity::day_index(created_at, now);
        let mut hasher = blake3::Hasher::new();
        hasher.update(peer_id.as_str().as_bytes());
        hasher.update(&created_at.as_secs().to_be_bytes());
        hasher.update(&day.to_be_bytes());
        let hash = hasher.finalize();
        let mut leading = [0_u8; 8];
        for (dst, src) in leading.iter_mut().zip(hash.as_bytes()) {
            *dst = *src;
        }
        let raw = u64::from_be_bytes(leading);
        let count = u64::try_from(starboard::identity::COMBINATION_COUNT).unwrap_or(72);
        let slot = usize::try_from(raw.checked_rem(count).unwrap_or(0)).unwrap_or(0);
        starboard::identity::brush_at(slot)
    }

    pub(crate) fn cursor_image(self, radius: u16) -> starboard::render::BrushImage {
        let edge = starboard::render::palette_rgba(self.color);
        let [red, green, blue, _] = edge;
        let fill = [red, green, blue, starboard::render::PREVIEW_FILL_ALPHA];
        starboard::render::brush_image(self.shape, radius, fill, edge)
    }

    pub(crate) fn icon_image(self, radius: u16) -> Image {
        let starboard::render::BrushImage { side, pixels } =
            starboard::render::solid_brush_image(self.shape, radius, self.color);
        starboard::render::square_image(side, pixels)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use freenet_libp2p_bevy_plugin::identity::PeerId;
    use freenet_libp2p_bevy_plugin::time::UnixTime;

    use super::DailyBrush;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let peer = PeerId::from("12D3KooWPeerAlpha");
        let created_at = UnixTime::from_secs(10_000);
        let same_day = DailyBrush::from_peer(&peer, created_at, UnixTime::from_secs(10_500));
        assert_eq!(
            same_day,
            DailyBrush::from_peer(&peer, created_at, UnixTime::from_secs(96_399))
        );
    }

    #[test]
    fn changes_across_rollover_and_covers_all_shapes_and_colors() {
        let created_at = UnixTime::from_secs(10_000);
        let day_zero = created_at;
        let day_one = UnixTime::from_secs(
            created_at
                .as_secs()
                .saturating_add(starboard::identity::SECONDS_PER_DAY),
        );
        let mut changed_on_rollover = 0_usize;
        let mut shapes_seen = BTreeSet::new();
        let mut colors_seen = BTreeSet::new();

        for i in 0_u32..10_000 {
            let peer = PeerId::from(format!("peer-{i}"));
            let first = DailyBrush::from_peer(&peer, created_at, day_zero);
            let second = DailyBrush::from_peer(&peer, created_at, day_one);
            if first != second {
                changed_on_rollover = changed_on_rollover.saturating_add(1);
            }
            shapes_seen.insert(first.shape);
            colors_seen.insert(first.color);
        }

        assert!(changed_on_rollover > 9_000);
        assert_eq!(shapes_seen.len(), starboard::identity::SHAPES.len());
        assert_eq!(colors_seen.len(), starboard::identity::PALETTE.len());
    }
}
