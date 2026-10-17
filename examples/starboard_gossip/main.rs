#[path = "../starboard/mod.rs"]
mod starboard;

fn main() {
    let now = freenet_libp2p_bevy_plugin::discovery::UnixTime::from_secs(0);
    let _ = starboard::identity::room_brushes(std::iter::empty(), now, now);
}
