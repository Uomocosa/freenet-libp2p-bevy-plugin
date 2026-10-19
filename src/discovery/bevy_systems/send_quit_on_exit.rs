use bevy::prelude::*;

use crate::discovery;
use discovery::{Command, CommandSender};

pub fn send_quit_on_exit(mut exits: MessageReader<AppExit>, sender: Res<CommandSender>) {
    if exits.read().next().is_some() {
        sender.into_inner().send(Command::Quit).ok();
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::send_quit_on_exit;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new();
        app.add_message::<AppExit>()
            .insert_resource(discovery::CommandSender(tx))
            .add_systems(Last, send_quit_on_exit);
        app.update();
        assert!(rx.try_recv().is_err());
        app.world_mut().write_message(AppExit::Success);
        app.update();
        assert_eq!(rx.try_recv(), Ok(discovery::Command::Quit));
    }
}
