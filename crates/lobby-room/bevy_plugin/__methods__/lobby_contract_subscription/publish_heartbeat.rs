use std::collections::BTreeMap;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
use freenet_stdlib::prelude::{StateDelta, UpdateData};

use crate::discovery;
use crate::identity;
use crate::time;

pub fn publish_heartbeat(
    state: &discovery::LobbyContractSubscription,
    room: &discovery::RoomName,
    me: &identity::PeerId,
    room_member: discovery::RoomMember,
    room_size: discovery::RoomSize,
    created_at: time::UnixTime,
) -> Result<(), discovery::Error> {
    let members = BTreeMap::from([(me.clone(), room_member)]);
    let update = discovery::Lobby::from([(
        room.clone(),
        discovery::Room {
            room_size,
            created_at,
            members,
        },
    )]);
    let request = ContractRequest::Update {
        key: state.key,
        data: UpdateData::Delta(StateDelta::from(bincode::serialize(&update)?)),
    };
    state.client.send(&ClientRequest::ContractOp(request))?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::publish_heartbeat;
    use crate::time;
    use crate::{discovery, identity, testing};

    #[test]
    fn test_usage() {
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let subscription = testing::lobby_subscription(write, read);
        let room_member =
            testing::room_member("me", None, discovery::RoomMembership::Occupant, &[]);
        assert!(
            publish_heartbeat(
                &subscription,
                &discovery::RoomName::from("room-a"),
                &identity::PeerId::from("me"),
                room_member,
                discovery::RoomSize::DEFAULT,
                time::UnixTime::EPOCH,
            )
            .is_ok()
        );
        assert!(write_rx.try_recv().is_ok());
    }
}
