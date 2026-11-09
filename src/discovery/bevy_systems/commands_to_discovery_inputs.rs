use bevy::prelude::*;

use crate::discovery;

pub fn commands_to_discovery_inputs(
    mut commands: MessageReader<discovery::Command>,
    mut inputs: MessageWriter<discovery::state_machine::Input>,
) {
    for command in commands.read() {
        inputs.write(discovery::state_machine::Input::Command(command.clone()));
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::commands_to_discovery_inputs;
    use crate::discovery;
    use crate::identity;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<discovery::Command>();
        app.add_message::<discovery::state_machine::Input>();
        app.add_systems(Update, commands_to_discovery_inputs);
        app.world_mut()
            .resource_mut::<Messages<discovery::Command>>()
            .write(discovery::Command::Join(identity::RoomName::from("room-a")));
        app.update();
        let inputs: Vec<discovery::state_machine::Input> = app
            .world_mut()
            .resource_mut::<Messages<discovery::state_machine::Input>>()
            .drain()
            .collect();
        assert_eq!(
            inputs,
            vec![discovery::state_machine::Input::Command(
                discovery::Command::Join(identity::RoomName::from("room-a"))
            )]
        );
    }

    #[test]
    fn no_command_writes_no_input() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<discovery::Command>();
        app.add_message::<discovery::state_machine::Input>();
        app.add_systems(Update, commands_to_discovery_inputs);
        app.update();
        let count = app
            .world_mut()
            .resource_mut::<Messages<discovery::state_machine::Input>>()
            .drain()
            .count();
        assert_eq!(count, 0);
    }
}
