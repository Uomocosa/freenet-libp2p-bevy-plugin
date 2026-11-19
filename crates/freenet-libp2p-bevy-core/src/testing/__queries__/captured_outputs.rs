use bevy::prelude::App;

use crate::p2p;
use crate::replica_sync;
use crate::testing;

#[must_use]
pub fn captured_outputs<Op: Clone + Send + Sync + 'static>(
    app: &App,
) -> (
    Vec<p2p::Libp2pCommand>,
    Vec<replica_sync::state_machine::Input<Op>>,
) {
    let captured = app.world().resource::<testing::CapturedOutputs<Op>>();
    (captured.commands.clone(), captured.inputs.clone())
}

#[cfg(test)]
mod tests {
    use bevy::prelude::App;

    use crate::replica_sync;
    use crate::testing;

    use super::captured_outputs;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.init_resource::<testing::CapturedOutputs<u8>>();
        app.world_mut()
            .resource_mut::<testing::CapturedOutputs<u8>>()
            .inputs
            .push(replica_sync::state_machine::Input::SnapshotMissingFromZoneMember);
        assert_eq!(captured_outputs::<u8>(&app).1.len(), 1);
    }
}
