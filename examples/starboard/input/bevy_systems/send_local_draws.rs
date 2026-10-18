use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::room_sync;

use crate::starboard;

pub fn send_local_draws(
    mut draws: MessageReader<starboard::input::PendingLocalDraw>,
    mut pending: ResMut<room_sync::PendingOps<starboard::canvas::Canvas>>,
) {
    for draw in draws.read() {
        pending.send(**draw);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::room_sync;

    use super::send_local_draws;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<room_sync::PendingOps<starboard::canvas::Canvas>>();
        app.add_message::<starboard::input::PendingLocalDraw>();
        app.add_systems(Update, send_local_draws);

        let op = starboard::canvas::DrawOp {
            action: starboard::canvas::DrawAction::Paint(starboard::identity::PaletteColor(5)),
            shape: starboard::identity::Shape::Circle,
            size_index: starboard::canvas::DEFAULT_SIZE_INDEX,
            from: starboard::canvas::CellPos { x: 50, y: 50 },
            to: starboard::canvas::CellPos { x: 50, y: 50 },
        };
        app.world_mut()
            .write_message(starboard::input::PendingLocalDraw(op));
        app.update();

        let pending = app
            .world()
            .resource::<room_sync::PendingOps<starboard::canvas::Canvas>>();
        assert_eq!(pending.ops.len(), 1);
        assert_eq!(
            pending.ops.first().map(|op| op.from),
            Some(starboard::canvas::CellPos { x: 50, y: 50 })
        );
    }
}
