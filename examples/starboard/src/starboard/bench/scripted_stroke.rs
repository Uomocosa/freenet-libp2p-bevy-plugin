use crate::starboard;

#[must_use]
pub fn scripted_stroke(
    peer_index: usize,
    peer_count: usize,
    tick: u32,
    brush: starboard::input::OwnBrush,
) -> starboard::canvas::DrawOp {
    let rows_per_peer = starboard::canvas::CANVAS_SIDE
        .checked_div(peer_count.max(1))
        .unwrap_or(0);
    let tick = usize::try_from(tick).unwrap_or(0);
    let row = peer_index
        .saturating_mul(rows_per_peer)
        .saturating_add(tick.checked_rem(rows_per_peer.max(1)).unwrap_or(0));
    let band = tick.checked_div(rows_per_peer.max(1)).unwrap_or(0);
    let x = band
        .saturating_mul(starboard::bench::STROKE_BAND_WIDTH)
        .saturating_add(starboard::bench::STROKE_START_OFFSET);
    let from = starboard::canvas::cell_centre(starboard::canvas::CellPos {
        x: u16::try_from(x).unwrap_or(0),
        y: u16::try_from(row).unwrap_or(0),
    });
    let to = starboard::canvas::cell_centre(starboard::canvas::CellPos {
        x: u16::try_from(x.saturating_add(starboard::bench::STROKE_LENGTH)).unwrap_or(u16::MAX),
        y: u16::try_from(row).unwrap_or(0),
    });
    let stroke_rank = u32::try_from(
        tick.saturating_mul(peer_count)
            .saturating_add(peer_index)
            .saturating_add(1),
    )
    .unwrap_or(u32::MAX);
    starboard::input::draw_op(
        brush,
        starboard::canvas::DrawAction::Paint(brush.color),
        starboard::canvas::Stroke { from, to },
        stroke_rank,
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::scripted_stroke;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let brush = starboard::input::OwnBrush::default();
        for peers in starboard::bench::BENCH_PEER_COUNTS {
            let mut seen = BTreeSet::new();
            for tick in 0..starboard::bench::BENCH_TICKS {
                for peer in 0..peers {
                    let op = scripted_stroke(peer, peers, tick, brush);
                    assert!(
                        seen.insert((op.from, op.to)),
                        "stroke repeated at peer {peer} tick {tick}"
                    );
                    let side = u32::try_from(starboard::canvas::CANVAS_SIDE).unwrap_or(0);
                    let sub_side = side.saturating_mul(starboard::canvas::SUBCELL_STEPS);
                    assert!(op.to.x < sub_side);
                    assert!(op.to.y < sub_side);
                    assert_eq!(op.from.y, op.to.y);
                }
            }
        }
    }
}
