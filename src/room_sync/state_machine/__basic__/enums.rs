use crate::identity;
use crate::room_sync;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    #[default]
    Idle,
    Fetching,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input<O> {
    Joined {
        room: identity::RoomName,
        members: Vec<identity::PeerId>,
    },
    Members {
        members: Vec<identity::PeerId>,
    },
    Remote {
        room: identity::RoomName,
        meta: room_sync::OpMeta,
        op: O,
    },
    Snapshot {
        room: identity::RoomName,
        part: u32,
        total: u32,
        data: Vec<u8>,
    },
    Fetch {
        id: u64,
        room: identity::RoomName,
        part: u32,
    },
    Tick,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output<O> {
    Fetch {
        peer: identity::PeerId,
        room: identity::RoomName,
        part: u32,
    },
    Snapshot {
        id: u64,
        room: identity::RoomName,
        part: u32,
    },
    Merge {
        bytes: Vec<u8>,
    },
    Apply {
        meta: room_sync::OpMeta,
        op: O,
    },
}
