use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use lobby_room::ping::RoomMessageSenders;
use serde_json::Value;

pub fn lobby_room_messages(
    In(_params): In<Option<Value>>,
    senders: Res<RoomMessageSenders>,
) -> BrpResult {
    let peers: Vec<String> = senders
        .into_inner()
        .peers
        .iter()
        .map(|peer| peer.as_str().to_owned())
        .collect();
    serde_json::to_value(peers).map_err(BrpError::internal)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::identity;
    use lobby_room::ping::RoomMessageSenders;
    use serde_json::Value;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        let mut senders = RoomMessageSenders::default();
        senders.peers.insert(identity::PeerId::from("peer-b"));
        app.insert_resource(senders);
        let system = app.world_mut().register_system(super::lobby_room_messages);
        let result = app.world_mut().run_system_with(system, None);
        let Ok(Ok(value)) = result else {
            panic!("messages did not run: {result:?}");
        };
        assert_eq!(value, Value::from(vec!["peer-b"]));
    }
}
