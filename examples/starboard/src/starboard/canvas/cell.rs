use serde::{Deserialize, Serialize};

use crate::starboard;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cell {
    pub top: starboard::canvas::Layer,
    pub under: Option<starboard::canvas::UnderLayer>,
}

impl Cell {
    pub(crate) fn absorb(&mut self, write: starboard::canvas::Layer) -> bool {
        if write.coverage == 0 {
            return false;
        }
        let write_key = write.key();
        let top_key = self.top.key();
        if write_key > top_key {
            if self.top.is_full() {
                self.under = Some(starboard::canvas::UnderLayer::from(self.top));
            }
            self.top = write;
            return true;
        }
        if write_key == top_key {
            let is_raised = write.coverage > self.top.coverage;
            if is_raised {
                self.top.coverage = write.coverage;
            }
            return is_raised;
        }
        let is_new_under =
            write.is_full() && self.under.is_none_or(|under| write_key > under.key());
        if is_new_under {
            self.under = Some(starboard::canvas::UnderLayer::from(write));
        }
        is_new_under
    }

    pub(crate) fn absorb_cell(&mut self, remote: &Self) -> bool {
        let is_top_changed = self.absorb(remote.top);
        let is_under_changed = remote
            .under
            .is_some_and(|under| self.absorb(under.as_full_layer()));
        is_top_changed || is_under_changed
    }
}

#[cfg(test)]
mod tests {
    use super::Cell;
    use crate::starboard;
    use starboard::canvas::{FULL_COVERAGE, Layer};
    use starboard::identity::PaletteColor;

    fn write(rank: u32, author: u16, color: Option<u8>, coverage: u8) -> Layer {
        Layer {
            rank,
            author,
            color: color.map(PaletteColor),
            coverage,
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
        let coverages = [FULL_COVERAGE, FULL_COVERAGE, 128, 64, 1];
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
                let coverage = coverages
                    .get(usize::try_from(next().checked_rem(5).unwrap_or(0)).unwrap_or(0))
                    .copied()
                    .unwrap_or(FULL_COVERAGE);
                write(rank, author, colour, coverage)
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

        let mut left = first;
        let _ = left.absorb_cell(&second);
        let _ = left.absorb_cell(&third);

        let mut right = third;
        let _ = right.absorb_cell(&first);
        let _ = right.absorb_cell(&second);

        assert_eq!(left, direct);
        assert_eq!(right, direct);
    }

    #[test]
    fn earlier_full_paint_becomes_under_when_a_later_edge_lands_on_it() {
        let mut cell = Cell::default();
        let _ = cell.absorb(write(10, 1, Some(0), FULL_COVERAGE));
        let _ = cell.absorb(write(11, 2, Some(6), 128));
        assert_eq!(cell.top.color, Some(PaletteColor(6)));
        assert_eq!(
            cell.under.map(|under| under.color),
            Some(Some(PaletteColor(0)))
        );
    }

    #[test]
    fn same_key_keeps_the_larger_coverage() {
        let mut cell = Cell::default();
        let _ = cell.absorb(write(3, 1, Some(2), 64));
        let _ = cell.absorb(write(3, 1, Some(2), 200));
        let _ = cell.absorb(write(3, 1, Some(2), 100));
        assert_eq!(cell.top.coverage, 200);
        assert_eq!(cell.under, None);
    }

    #[test]
    fn test_usage() {
        let mut cell = Cell::default();
        assert!(cell.absorb(write(1, 1, Some(4), FULL_COVERAGE)));
        assert!(!cell.absorb(write(0, 1, Some(4), 10)));
    }
}
