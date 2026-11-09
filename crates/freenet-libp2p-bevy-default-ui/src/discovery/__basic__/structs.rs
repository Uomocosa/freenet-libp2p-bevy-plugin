use freenet_libp2p_bevy_core::discovery;
use freenet_libp2p_bevy_core::identity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomRow {
    pub name: identity::RoomName,
    pub members: usize,
    pub room_size: discovery::RoomSize,
}
