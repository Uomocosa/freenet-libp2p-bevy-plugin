use crate::net_id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomDetails {
    pub name: net_id::RoomName,
    pub members: usize,
    pub capacity: u16,
}
