use serde::{Deserialize, Serialize};

use crate::starboard;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cell {
    pub top: starboard::canvas::Layer,
    pub under: Option<starboard::canvas::UnderLayer>,
    pub lower: Box<[starboard::canvas::Layer]>,
}

impl Cell {
    pub(crate) fn entries(&self) -> Vec<starboard::canvas::Layer> {
        let mut entries = self.lower.to_vec();
        if self.top.mask != 0 {
            entries.push(self.top);
        }
        if let Some(under) = self.under {
            entries.push(under.as_full_layer());
        }
        entries
    }

    pub(crate) fn rebuild(&mut self, mut entries: Vec<starboard::canvas::Layer>) {
        entries.sort_by_key(starboard::canvas::Layer::key);
        let top = entries.pop().unwrap_or_default();
        let under_index = entries.iter().rposition(starboard::canvas::Layer::is_full);
        self.under = under_index
            .and_then(|index| entries.get(index))
            .map(|layer| starboard::canvas::UnderLayer::from(*layer));
        let floor = self.under.map(super::UnderLayer::key);
        self.lower = entries
            .into_iter()
            .filter(|layer| floor.is_none_or(|key| layer.key() > key))
            .collect();
        self.top = top;
    }

    pub(crate) fn absorb(&mut self, write: starboard::canvas::Layer) -> bool {
        if write.mask == 0 {
            return false;
        }
        let before = self.clone();
        let mut entries = self.entries();
        match entries.iter_mut().find(|layer| layer.key() == write.key()) {
            Some(layer) => layer.mask |= write.mask,
            None => entries.push(write),
        }
        self.rebuild(entries);
        *self != before
    }

    pub(crate) fn absorb_cell(&mut self, remote: &Self) -> bool {
        let mut is_changed = false;
        for layer in remote.entries() {
            if self.absorb(layer) {
                is_changed = true;
            }
        }
        is_changed
    }
}

#[cfg(test)]
mod tests {
    use super::Cell;
    use crate::starboard;
    use starboard::canvas::{FULL_MASK, Layer};
    use starboard::identity::PaletteColor;

    fn write(rank: u32, author: u16, color: Option<u8>, mask: u16) -> Layer {
        Layer {
            rank,
            author,
            color: color.map(PaletteColor),
            mask,
        }
    }

    fn seeded_writes(count: u64) -> Vec<Layer> {
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next = move || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            state >> 33
        };
        let masks = [FULL_MASK, FULL_MASK, 0x00FF, 0x0F0F, 0x0001, 0xF000];
        (0..count)
            .map(|_| {
                let rank = u32::try_from(next().checked_rem(6).unwrap_or(0))
                    .unwrap_or(0)
                    .saturating_add(1);
                let author = u16::try_from(next().checked_rem(3).unwrap_or(0))
                    .unwrap_or(0)
                    .saturating_add(1);
                let colour = match next().checked_rem(4).unwrap_or(0) {
                    0 => None,
                    other => Some(u8::try_from(other).unwrap_or(0)),
                };
                let mask = masks
                    .get(usize::try_from(next().checked_rem(6).unwrap_or(0)).unwrap_or(0))
                    .copied()
                    .unwrap_or(FULL_MASK);
                write(rank, author, colour, mask)
            })
            .collect()
    }

    fn cell_of(writes: &[Layer]) -> Cell {
        let mut cell = Cell::default();
        for layer in writes {
            let _ = cell.absorb(*layer);
        }
        cell
    }

    fn shuffled(writes: &[Layer], seed: usize) -> Vec<Layer> {
        let mut order = writes.to_vec();
        let len = order.len();
        for index in (1..len).rev() {
            let other = seed
                .wrapping_mul(31)
                .wrapping_add(index.wrapping_mul(17))
                .checked_rem(index.saturating_add(1))
                .unwrap_or(0);
            order.swap(index, other);
        }
        order
    }

    #[test]
    fn merge_is_order_independent() {
        let writes = seeded_writes(200);
        let reference = cell_of(&writes);
        let reversed: Vec<Layer> = writes.iter().rev().copied().collect();
        let rotated: Vec<Layer> = writes
            .iter()
            .cycle()
            .skip(97)
            .take(writes.len())
            .copied()
            .collect();
        for order in [
            reversed,
            rotated,
            shuffled(&writes, 3),
            shuffled(&writes, 11),
        ] {
            assert_eq!(cell_of(&order), reference);
        }
    }

    #[test]
    fn three_way_split_merge_equals_applying_every_write() {
        let writes = seeded_writes(200);
        let direct = cell_of(&writes);
        let first = cell_of(writes.get(..70).unwrap_or_default());
        let second = cell_of(writes.get(70..140).unwrap_or_default());
        let third = cell_of(writes.get(140..).unwrap_or_default());

        let mut left = first.clone();
        let _ = left.absorb_cell(&second);
        let _ = left.absorb_cell(&third);

        let mut right = third;
        let _ = right.absorb_cell(&first);
        let _ = right.absorb_cell(&second);

        assert_eq!(left, direct);
        assert_eq!(right, direct);
    }

    #[test]
    fn lower_partials_survive_a_higher_top_and_complete_later() {
        let low_first = write(3, 1, Some(0), 0x00FF);
        let low_second = write(3, 1, Some(0), 0xFF00);
        let high = write(9, 2, Some(6), FULL_MASK);

        let mut early = Cell::default();
        let _ = early.absorb(low_first);
        let _ = early.absorb(high);
        let _ = early.absorb(low_second);

        let mut late = Cell::default();
        let _ = late.absorb(low_first);
        let _ = late.absorb(low_second);
        let _ = late.absorb(high);

        assert_eq!(early, late);
        assert_eq!(
            early.under.map(|under| under.color),
            Some(Some(PaletteColor(0)))
        );
    }

    #[test]
    fn earlier_full_paint_becomes_under_when_a_later_edge_lands_on_it() {
        let mut cell = Cell::default();
        let _ = cell.absorb(write(10, 1, Some(0), FULL_MASK));
        let _ = cell.absorb(write(11, 2, Some(6), 0x00FF));
        assert_eq!(cell.top.color, Some(PaletteColor(6)));
        assert_eq!(
            cell.under.map(|under| under.color),
            Some(Some(PaletteColor(0)))
        );
    }

    #[test]
    fn same_key_ors_the_masks() {
        let mut cell = Cell::default();
        let _ = cell.absorb(write(3, 1, Some(2), 0x000F));
        let _ = cell.absorb(write(3, 1, Some(2), 0x00F0));
        let _ = cell.absorb(write(3, 1, Some(2), 0x000F));
        assert_eq!(cell.top.mask, 0x00FF);
        assert_eq!(cell.under, None);
        let _ = cell.absorb(write(3, 1, Some(2), 0xFF00));
        assert_eq!(cell.top.mask, FULL_MASK);
        assert_eq!(cell.under, None);
    }

    #[test]
    fn test_usage() {
        let mut cell = Cell::default();
        assert!(cell.absorb(write(1, 1, Some(4), FULL_MASK)));
        assert!(cell.absorb(write(0, 1, Some(4), 0x0010)));
        assert_eq!(cell.lower.len(), 1);
    }
}
