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

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::decode_extension_effects;
    use crate::discovery;
    use crate::identity;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<testing::E2EExtensionEffect>();
        app.add_message::<discovery::Command>();
        app.add_systems(Update, decode_extension_effects);
        let command = discovery::Command::Create(identity::RoomName::from("room-1"));
        app.world_mut()
            .resource_mut::<Messages<testing::E2EExtensionEffect>>()
            .write(testing::E2EExtensionEffect(command.to_extension_bytes()));
        app.update();
        let decoded: Vec<discovery::Command> = app
            .world_mut()
            .resource_mut::<Messages<discovery::Command>>()
            .drain()
            .collect();
        assert_eq!(decoded, vec![command]);
    }
}
