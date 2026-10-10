use bevy::prelude::*;

use crate::discovery;
use discovery::{Command, CommandSender};

pub fn forward_commands(mut commands: MessageReader<Command>, sender: Res<CommandSender>) {
    let sender = sender.into_inner();
    for command in commands.read() {
        sender.send(command.clone()).ok();
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::forward_commands;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<discovery::Command>();
        app.insert_resource(discovery::CommandSender(tx));
        app.add_systems(Update, forward_commands);
        app.world_mut()
            .resource_mut::<Messages<discovery::Command>>()
            .write(discovery::Command::Leave);
        app.update();
        assert_eq!(rx.try_recv(), Ok(discovery::Command::Leave));
    }
}
