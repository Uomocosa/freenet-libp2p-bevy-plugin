use crate::identity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomDetails {
    pub name: identity::RoomName,
    pub members: usize,
    pub room_size: u16,
}
