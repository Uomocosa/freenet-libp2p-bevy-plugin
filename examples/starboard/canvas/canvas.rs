use std::collections::BTreeSet;

use bevy::prelude::Resource;
use freenet_libp2p_bevy_plugin::room_sync;

use crate::starboard;

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct Canvas {
    pub tiles: [Option<Box<starboard::canvas::Tile>>; starboard::canvas::TILE_COUNT],
    pub dirty_tiles: BTreeSet<starboard::canvas::TileId>,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            tiles: std::array::from_fn(|_| None),
            dirty_tiles: BTreeSet::new(),
        }
    }
}

impl room_sync::RoomState for Canvas {
    type Op = starboard::canvas::DrawOp;
    type Snapshot = starboard::canvas::CanvasSnapshot;

    fn apply(&mut self, meta: room_sync::OpMeta, op: &Self::Op) {
        let color = match op.action {
            starboard::canvas::DrawAction::Paint(palette_color) => Some(palette_color),
            starboard::canvas::DrawAction::Erase => None,
        };
        let cell = starboard::canvas::Cell {
            lamport: meta.lamport,
            author: meta.author,
            color,
        };
        for pos in starboard::canvas::rasterize(op) {
            let _ = starboard::canvas::write_cell(self, pos, cell);
        }
    }

    fn snapshot(&self) -> Self::Snapshot {
        let mut touched = Vec::new();
        for (idx, maybe_tile) in self.tiles.iter().enumerate() {
            if let Some(tile) = maybe_tile {
                touched.push(starboard::canvas::TouchedTile {
                    id: starboard::canvas::TileId(u16::try_from(idx).unwrap_or(0)),
                    cells: tile.iter().copied().collect(),
                });
            }
        }
        starboard::canvas::encode_snapshot(&touched)
    }

    fn merge(&mut self, snapshot: Self::Snapshot) {
        let touched = starboard::canvas::decode_snapshot(&snapshot);
        for entry in touched {
            if let Some(tile) = tile_from_cells(&entry.cells) {
                starboard::canvas::merge_tile(self, entry.id, &tile);
            }
        }
    }

    fn slice_content_hashes(&self) -> room_sync::ReplicaSliceContentHashes {
        let mut hashes = Vec::new();
        for (idx, maybe_tile) in self.tiles.iter().enumerate() {
            if let Some(tile) = maybe_tile {
                let slice_idx = room_sync::ReplicaSliceIndex(u32::try_from(idx).unwrap_or(0));
                hashes.push((slice_idx, starboard::canvas::hash_tile(tile)));
            }
        }
        hashes
    }

    fn slice_snapshot(&self, index: room_sync::ReplicaSliceIndex) -> Self::Snapshot {
        let idx = usize::try_from(*index).unwrap_or(starboard::canvas::TILE_COUNT);
        let mut touched = Vec::new();
        if let Some(Some(tile)) = self.tiles.get(idx) {
            touched.push(starboard::canvas::TouchedTile {
                id: starboard::canvas::TileId(u16::try_from(idx).unwrap_or(0)),
                cells: tile.iter().copied().collect(),
            });
        }
        starboard::canvas::encode_snapshot(&touched)
    }
}

// needed helper: convert a decoded Vec<Cell> back into a fixed-size Tile if its length matches
fn tile_from_cells(cells: &[starboard::canvas::Cell]) -> Option<starboard::canvas::Tile> {
    if cells.len() != starboard::canvas::CELLS_PER_TILE {
        return None;
    }
    let mut tile = starboard::canvas::Tile::default();
    for (dst, src) in tile.iter_mut().zip(cells.iter()) {
        *dst = *src;
    }
    Some(tile)
}

#[cfg(test)]
mod tests {
    use freenet_libp2p_bevy_plugin::room_sync::{OpMeta, ReplicaSliceIndex, RoomState};

    use super::Canvas;
    use crate::starboard;

    fn sample_op(seed: u64) -> (OpMeta, starboard::canvas::DrawOp) {
        let mut leading = [0_u8; 8];
        let hash = blake3::hash(&seed.to_be_bytes());
        for (dst, src) in leading.iter_mut().zip(hash.as_bytes()) {
            *dst = *src;
        }
        let raw = u64::from_be_bytes(leading);
        let lamport = u32::try_from((raw & 0xFFFF).saturating_add(1)).unwrap_or(1);
        let author = u16::try_from((raw >> 16) & 0xFFFF).unwrap_or(1);
        let from_x = u16::try_from((raw >> 32) % 1024).unwrap_or(0);
        let from_y = u16::try_from((raw >> 42) % 1024).unwrap_or(0);
        let dx = u16::try_from((raw >> 52) % 24).unwrap_or(0);
        let dy = u16::try_from((raw >> 58) % 24).unwrap_or(0);
        let to_x = from_x.saturating_add(dx).min(1023);
        let to_y = from_y.saturating_add(dy).min(1023);
        let color_idx = u8::try_from(raw % 13).unwrap_or(0);
        let action = if color_idx == 12 {
            starboard::canvas::DrawAction::Erase
        } else {
            starboard::canvas::DrawAction::Paint(starboard::identity::PaletteColor(color_idx))
        };
        let shape_idx = usize::try_from((raw >> 8) % 6).unwrap_or(0);
        let shape = starboard::identity::SHAPES
            .get(shape_idx)
            .copied()
            .unwrap_or(starboard::identity::Shape::Circle);
        (
            OpMeta { lamport, author },
            starboard::canvas::DrawOp {
                action,
                shape,
                size_index: 5,
                from: starboard::canvas::CellPos {
                    x: from_x,
                    y: from_y,
                },
                to: starboard::canvas::CellPos { x: to_x, y: to_y },
            },
        )
    }

    #[test]
    fn test_usage() {
        let mut a = Canvas::default();
        let mut b = Canvas::default();
        let (meta_1, op_1) = sample_op(1);
        let (meta_2, op_2) = sample_op(2);
        a.apply(meta_1, &op_1);
        b.apply(meta_2, &op_2);

        let snap_a = a.snapshot();
        let snap_b = b.snapshot();
        let mut ab = a.clone();
        ab.merge(snap_b);
        let mut ba = b.clone();
        ba.merge(snap_a);

        assert_ne!(starboard::canvas::hash_canvas(&ab), 0);
        assert_eq!(
            starboard::canvas::hash_canvas(&ab),
            starboard::canvas::hash_canvas(&ba)
        );
        let before_idempotent = starboard::canvas::hash_canvas(&ab);
        let self_snap = ab.snapshot();
        ab.merge(self_snap);
        assert_eq!(starboard::canvas::hash_canvas(&ab), before_idempotent);
    }

    #[test]
    fn merge_is_associative_and_permutations_yield_identical_hash() {
        let ops: Vec<_> = (1_u64..=64).map(sample_op).collect();
        let mut forward = Canvas::default();
        for (meta, op) in &ops {
            forward.apply(*meta, op);
        }
        let mut reverse = Canvas::default();
        for (meta, op) in ops.iter().rev() {
            reverse.apply(*meta, op);
        }
        let mut interleaved = Canvas::default();
        for (idx, (meta, op)) in ops.iter().enumerate() {
            if idx % 2 == 0 {
                interleaved.apply(*meta, op);
            }
        }
        for (idx, (meta, op)) in ops.iter().enumerate() {
            if idx % 2 == 1 {
                interleaved.apply(*meta, op);
            }
        }
        assert_eq!(
            starboard::canvas::hash_canvas(&forward),
            starboard::canvas::hash_canvas(&reverse)
        );
        assert_eq!(
            starboard::canvas::hash_canvas(&forward),
            starboard::canvas::hash_canvas(&interleaved)
        );

        let mut c1 = Canvas::default();
        let mut c2 = Canvas::default();
        let mut c3 = Canvas::default();
        for (meta, op) in ops.iter().take(20) {
            c1.apply(*meta, op);
        }
        for (meta, op) in ops.iter().skip(20).take(20) {
            c2.apply(*meta, op);
        }
        for (meta, op) in ops.iter().skip(40) {
            c3.apply(*meta, op);
        }
        let mut left = c1.clone();
        left.merge(c2.snapshot());
        left.merge(c3.snapshot());

        let mut right_inner = c2;
        right_inner.merge(c3.snapshot());
        let mut right = c1;
        right.merge(right_inner.snapshot());

        assert_eq!(
            starboard::canvas::hash_canvas(&left),
            starboard::canvas::hash_canvas(&right)
        );
        assert_eq!(left.slice_content_hashes(), right.slice_content_hashes());
        if let Some((first_slice, _)) = left.slice_content_hashes().first().copied() {
            let mut repaired = Canvas::default();
            repaired.merge(left.slice_snapshot(first_slice));
            assert_eq!(repaired.slice_content_hashes().len(), 1);
            assert!(
                repaired.slice_snapshot(ReplicaSliceIndex(9999)).len()
                    <= left.slice_snapshot(first_slice).len()
            );
        }
    }

    #[test]
    fn snapshot_sizes_for_1k_and_10k_random_strokes() {
        for stroke_count in [1_000_u64, 10_000_u64] {
            let mut canvas = Canvas::default();
            for seed in 1..=stroke_count {
                let (meta, op) = sample_op(seed);
                canvas.apply(meta, &op);
            }
            let touched_tiles: Vec<_> = canvas
                .tiles
                .iter()
                .enumerate()
                .filter_map(|(idx, maybe_tile)| {
                    maybe_tile
                        .as_ref()
                        .map(|tile| starboard::canvas::TouchedTile {
                            id: starboard::canvas::TileId(u16::try_from(idx).unwrap_or(0)),
                            cells: tile.iter().copied().collect(),
                        })
                })
                .collect();
            let raw_bytes = touched_tiles
                .len()
                .saturating_mul(starboard::canvas::CELLS_PER_TILE)
                .saturating_mul(std::mem::size_of::<starboard::canvas::Cell>());
            let bincode_bytes = bincode::serialize(&touched_tiles)
                .map(|bytes| bytes.len())
                .unwrap_or(0);
            let compressed = canvas.snapshot();
            println!(
                "strokes={stroke_count} touched_tiles={} raw_bytes={raw_bytes} bincode_bytes={bincode_bytes} zstd_bytes={}",
                touched_tiles.len(),
                compressed.len()
            );
            assert!(!touched_tiles.is_empty());
            assert!(!compressed.is_empty());
            assert!(compressed.len() < bincode_bytes);
        }
    }
}
