use crate::starboard;

#[must_use]
pub fn scripted_stroke(
    peer_index: usize,
    peer_count: usize,
    tick: u32,
    brush: starboard::input::OwnBrush,
) -> starboard::canvas::DrawOp {
    let rows_per_peer = starboard::canvas::CANVAS_SIDE / peer_count.max(1);
    let tick = usize::try_from(tick).unwrap_or(0);
    let row = peer_index
        .saturating_mul(rows_per_peer)
        .saturating_add(tick % rows_per_peer.max(1));
    let band = tick / rows_per_peer.max(1);
    let x = band
        .saturating_mul(starboard::bench::STROKE_BAND_WIDTH)
        .saturating_add(starboard::bench::STROKE_START_OFFSET);
    let from = starboard::canvas::CellPos {
        x: u16::try_from(x).unwrap_or(0),
        y: u16::try_from(row).unwrap_or(0),
    };
    let to = starboard::canvas::CellPos {
        x: u16::try_from(x.saturating_add(starboard::bench::STROKE_LENGTH)).unwrap_or(u16::MAX),
        y: from.y,
    };
    starboard::input::draw_op(
        brush,
        starboard::canvas::DrawAction::Paint(brush.color),
        starboard::canvas::Stroke { from, to },
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
                    assert!(usize::from(op.to.x) < starboard::canvas::CANVAS_SIDE);
                    assert!(usize::from(op.to.y) < starboard::canvas::CANVAS_SIDE);
                    assert_eq!(op.from.y, op.to.y);
                }
            }
        }
    }
}
