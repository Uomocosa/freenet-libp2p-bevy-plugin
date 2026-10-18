#[path = "../starboard/mod.rs"]
mod starboard;

fn main() {
    let now = freenet_libp2p_bevy_plugin::discovery::UnixTime::from_secs(0);
    let _ = starboard::identity::room_brushes(std::iter::empty(), now, now);
    let mut canvas = starboard::canvas::Canvas::default();
    let pos = starboard::canvas::CellPos::default();
    let _ = starboard::canvas::read_cell(&canvas, pos);
    let _ = starboard::canvas::write_cell(&mut canvas, pos, starboard::canvas::Cell::default());
    let _ = starboard::canvas::hash_canvas(&canvas);
    let op = starboard::canvas::DrawOp {
        action: starboard::canvas::DrawAction::Erase,
        shape: starboard::identity::Shape::Circle,
        size_index: starboard::canvas::DEFAULT_SIZE_INDEX,
        from: pos,
        to: pos,
    };
    let _ = starboard::canvas::rasterize(&op)
        .count()
        .saturating_add(usize::from(starboard::canvas::SIZE_PERCENTS[0]));
}
