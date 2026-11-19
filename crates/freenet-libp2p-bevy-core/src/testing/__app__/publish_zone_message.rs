use bevy::prelude::App;

use crate::identity;
use crate::p2p;

pub fn publish_zone_message(app: &App, topic: &identity::Topic, data: &[u8]) {
    let _ = app
        .world()
        .resource::<p2p::Libp2pCommandSender>()
        .send(p2p::Libp2pCommand::Publish {
            topic: topic.clone(),
            data: data.to_vec(),
        });
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::p2p;
    use crate::testing;

    use super::publish_zone_message;

    #[test]
    fn test_usage() {
        let mut app = testing::TestApp::new();
        let (sender, mut commands) = tokio::sync::mpsc::unbounded_channel();
        app.insert_resource(p2p::Libp2pCommandSender(sender));
        let topic = identity::Topic::from("topic");
        publish_zone_message(&app, &topic, &[7]);
        assert!(matches!(
            commands.try_recv(),
            Ok(p2p::Libp2pCommand::Publish { topic: got, data }) if got == topic && data == vec![7]
        ));
    }
}
