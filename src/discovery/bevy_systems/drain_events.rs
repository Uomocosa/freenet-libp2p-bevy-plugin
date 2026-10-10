use bevy::prelude::*;

use crate::discovery;
use discovery::{Event, EventFeed};

pub fn drain_events(feed: Res<EventFeed>, mut events: MessageWriter<Event>) {
    let feed = feed.into_inner();
    let Ok(mut guard) = feed.lock() else {
        return;
    };
    let Some(rx) = guard.as_mut() else {
        return;
    };
    while let Ok(event) = rx.try_recv() {
        events.write(event);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use bevy::prelude::*;

    use super::drain_events;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<discovery::Event>();
        app.insert_resource(discovery::EventFeed(Mutex::new(Some(rx))));
        app.add_systems(Update, drain_events);
        tx.send(discovery::Event::LobbyChanged).ok();
        app.update();
        let mut messages = app.world_mut().resource_mut::<Messages<discovery::Event>>();
        assert_eq!(messages.drain().count(), 1);
    }
}
