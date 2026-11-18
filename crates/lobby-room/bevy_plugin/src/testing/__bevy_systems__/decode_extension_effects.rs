use bevy::prelude::*;

use crate::discovery;
use crate::testing;

pub fn decode_extension_effects(
    mut effects: MessageReader<testing::E2EExtensionEffect>,
    mut commands: MessageWriter<discovery::Command>,
) {
    for effect in effects.read() {
        match bincode::deserialize::<discovery::Command>(effect) {
            Ok(command) => {
                commands.write(command);
            }
            Err(error) => {
                tracing::warn!(target: "script", "dropping undecodable extension effect: {error}");
            }
        }
    }
}
