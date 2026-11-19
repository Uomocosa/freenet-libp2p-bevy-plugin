use bevy::ecs::system::{RunSystemError, RunSystemOnce};
use bevy::prelude::{App, Message, MessageReader};

pub fn written_messages<M: Message + Clone>(app: &mut App) -> Result<Vec<M>, RunSystemError> {
    app.world_mut()
        .run_system_once(|mut reader: MessageReader<M>| reader.read().cloned().collect::<Vec<M>>())
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{App, Messages};

    use crate::testing;

    use super::written_messages;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<testing::E2EExtensionEffect>();
        app.world_mut()
            .resource_mut::<Messages<testing::E2EExtensionEffect>>()
            .write(testing::E2EExtensionEffect(vec![1]));
        assert_eq!(
            written_messages::<testing::E2EExtensionEffect>(&mut app).expect("written"),
            vec![testing::E2EExtensionEffect(vec![1])]
        );
    }
}
