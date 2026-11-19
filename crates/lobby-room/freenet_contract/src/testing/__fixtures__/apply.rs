use freenet_stdlib::prelude::ContractInterface;
use freenet_stdlib::prelude::Parameters;
use freenet_stdlib::prelude::StateDelta;
use freenet_stdlib::prelude::UpdateData;

use crate::Lobby;
use crate::LobbyContract;
use crate::testing;

#[must_use]
pub fn apply(params: &Parameters<'static>, base: &Lobby, incoming: &Lobby) -> Lobby {
    let update = UpdateData::Delta(StateDelta::from(
        bincode::serialize(incoming).expect("a lobby serializes"),
    ));
    let merged = LobbyContract::update_state(params.clone(), testing::state_of(base), vec![update])
        .expect("a valid update merges");
    bincode::deserialize(&testing::state_bytes(merged)).expect("the merged lobby decodes")
}

#[cfg(test)]
mod tests {
    use super::apply;
    use crate::Lobby;
    use crate::RoomName;
    use crate::testing;
    use common_types::UnixTime;
    use freenet_stdlib::prelude::Parameters;

    #[test]
    fn test_usage() {
        let params = Parameters::from(Vec::new());
        let mut incoming = Lobby::default();
        incoming.insert(
            RoomName::from("room-a"),
            testing::room("peer", UnixTime::from_secs(5), UnixTime::from_secs(1)),
        );
        let merged = apply(&params, &Lobby::default(), &incoming);
        assert_eq!(merged.len(), 1);
        assert!(merged.contains_key(&RoomName::from("room-a")));
    }
}
