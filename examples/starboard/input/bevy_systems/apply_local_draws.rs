use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::room_sync::{self, RoomState};

use crate::starboard;

pub fn apply_local_draws(
    mut draws: MessageReader<starboard::input::PendingLocalDraw>,
    mut canvas: ResMut<starboard::canvas::Canvas>,
    mut clock: ResMut<room_sync::LamportClock>,
) {
    for draw in draws.read() {
        let meta = room_sync::OpMeta {
            lamport: clock.tick(),
            author: 0,
        };
        canvas.apply(meta, &**draw);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::room_sync;

    use super::apply_local_draws;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<starboard::canvas::Canvas>();
        app.init_resource::<room_sync::LamportClock>();
        app.add_message::<starboard::input::PendingLocalDraw>();
        app.add_systems(Update, apply_local_draws);

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

        let canvas = app.world().resource::<starboard::canvas::Canvas>();
        let cell =
            starboard::canvas::read_cell(canvas, starboard::canvas::CellPos { x: 50, y: 50 });
        assert_eq!(cell.color, Some(starboard::identity::PaletteColor(5)));
        assert_eq!(cell.lamport, 1);
        assert_eq!(cell.author, 0);
    }
}
