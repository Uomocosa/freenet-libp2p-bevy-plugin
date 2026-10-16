use std::collections::BTreeMap;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
use freenet_stdlib::prelude::{StateDelta, UpdateData};

use crate::discovery;
use crate::identity;

pub fn publish_heartbeat(
    lobby_contract: &mut discovery::freenet::LobbyContract,
    room: &identity::RoomName,
    me: &identity::PeerId,
    heartbeat: discovery::Heartbeat,
    room_size: u16,
) -> Result<(), discovery::Error> {
    let members = BTreeMap::from([(me.clone(), heartbeat)]);
    let update = discovery::Lobby::from([(room.clone(), discovery::Room { room_size, members })]);
    let request = ContractRequest::Update {
        key: lobby_contract.key,
        data: UpdateData::Delta(StateDelta::from(bincode::serialize(&update)?)),
    };
    lobby_contract
        .client
        .send(&ClientRequest::ContractOp(request))?;
    Ok(())
}

// no test_usage necessary
