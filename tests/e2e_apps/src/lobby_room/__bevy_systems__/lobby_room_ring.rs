use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use serde_json::Value;

use crate::lobby_room::RingPeers;

pub fn lobby_room_ring(In(_params): In<Option<Value>>, ring: Res<RingPeers>) -> BrpResult {
    serde_json::to_value(ring.into_inner().get()).map_err(BrpError::internal)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use serde_json::Value;

    use crate::lobby_room::RingPeers;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        let ring = RingPeers::new();
        ring.set(4);
        app.insert_resource(ring);
        let system = app.world_mut().register_system(super::lobby_room_ring);
        let result = app.world_mut().run_system_with(system, None);
        let Ok(Ok(value)) = result else {
            panic!("ring did not run: {result:?}");
        };
        assert_eq!(value, Value::from(4));
    }
}
