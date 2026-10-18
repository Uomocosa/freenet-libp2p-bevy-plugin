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
}
