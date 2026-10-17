use serde::{Deserialize, Serialize};

use crate::identity;
use crate::room_sync;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplicaSyncTopicMessage<Op> {
    ApplyThisOp {
        room: identity::RoomName,
        meta: room_sync::OpMeta,
        op: Op,
    },
}
