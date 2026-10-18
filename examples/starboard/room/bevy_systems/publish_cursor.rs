use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::p2p;

use crate::starboard;

pub fn publish_cursor(
    time: Res<Time>,
    cursor: Res<starboard::input::CursorCell>,
    brush: Res<starboard::input::OwnBrush>,
    commands: Option<ResMut<p2p::Commands<starboard::room::CursorUpdate>>>,
    mut accumulator: Local<f32>,
) {
    let time = time.into_inner();
    let cursor = cursor.into_inner();
    let brush = brush.into_inner();
    let Some(cell) = **cursor else {
        return;
    };
    let Some(mut commands) = commands else {
        return;
    };
    *accumulator += time.delta_secs();
    if *accumulator < starboard::room::CURSOR_UPDATE_SECONDS {
        return;
    }
    *accumulator = 0.0;
    commands.push(p2p::Command::PublishInJoinedRoom {
        payload: starboard::room::CursorUpdate {
            cell,
            size_index: brush.size_index,
        },
    });
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;
    use freenet_libp2p_bevy_plugin::p2p;

    use super::publish_cursor;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )));
        app.insert_resource(starboard::input::CursorCell(Some(
            starboard::canvas::CellPos { x: 7, y: 9 },
        )));
        app.insert_resource(starboard::input::OwnBrush::default());
        app.insert_resource(p2p::Commands::<starboard::room::CursorUpdate>::default());
        app.add_systems(Update, publish_cursor);

        app.update();
        app.update();
        let commands = app
            .world()
            .resource::<p2p::Commands<starboard::room::CursorUpdate>>();
        assert_eq!(commands.len(), 1);
        assert!(matches!(
            commands.first(),
            Some(p2p::Command::PublishInJoinedRoom { payload })
                if payload.cell == starboard::canvas::CellPos { x: 7, y: 9 }
        ));

        app.world_mut()
            .resource_mut::<starboard::input::CursorCell>()
            .0 = None;
        app.update();
        assert_eq!(
            app.world()
                .resource::<p2p::Commands<starboard::room::CursorUpdate>>()
                .len(),
            1
        );
    }
}
