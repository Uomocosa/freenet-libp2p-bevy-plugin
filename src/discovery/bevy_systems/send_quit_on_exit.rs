use bevy::prelude::*;

use crate::discovery;

pub fn send_quit_on_exit(
    mut exits: MessageReader<AppExit>,
    mut inputs: MessageWriter<discovery::state_machine::Input>,
) {
    if exits.read().next().is_some() {
        inputs.write(discovery::state_machine::Input::Command(
            discovery::Command::Quit,
        ));
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::send_quit_on_exit;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<AppExit>();
        app.add_message::<discovery::state_machine::Input>();
        app.add_systems(Last, send_quit_on_exit);
        app.update();
        let count = app
            .world_mut()
            .resource_mut::<Messages<discovery::state_machine::Input>>()
            .drain()
            .count();
        assert_eq!(count, 0);
        app.world_mut().write_message(AppExit::Success);
        app.update();
        let inputs: Vec<discovery::state_machine::Input> = app
            .world_mut()
            .resource_mut::<Messages<discovery::state_machine::Input>>()
            .drain()
            .collect();
        assert_eq!(
            inputs,
            vec![discovery::state_machine::Input::Command(
                discovery::Command::Quit
            )]
        );
    }
}
