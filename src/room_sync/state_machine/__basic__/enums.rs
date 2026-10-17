use crate::identity;
use crate::p2p;
use crate::room_sync;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    #[default]
    Idle,
    Fetching,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input<Op> {
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
        op: Op,
    },
    Snapshot {
        room: identity::RoomName,
        part: u32,
        total: u32,
        data: Vec<u8>,
    },
    Fetch {
        id: p2p::ReplyId,
        room: identity::RoomName,
        part: u32,
    },
    Tick,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output<Op> {
    Fetch {
        peer: identity::PeerId,
        room: identity::RoomName,
        part: u32,
    },
    Snapshot {
        id: p2p::ReplyId,
        room: identity::RoomName,
        part: u32,
    },
    Merge {
        bytes: Vec<u8>,
    },
    Apply {
        meta: room_sync::OpMeta,
        op: Op,
    },
}
