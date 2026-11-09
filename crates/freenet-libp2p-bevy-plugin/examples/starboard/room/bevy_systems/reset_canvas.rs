use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::discovery;

use crate::starboard;

pub fn reset_canvas(
    mut joins: MessageReader<discovery::Event>,
    mut canvas: ResMut<starboard::canvas::Canvas>,
    mut latest_painted: ResMut<starboard::input::LatestPaintedCell>,
) {
    for event in joins.read() {
        if matches!(event, discovery::Event::Joined(_)) {
            *canvas = starboard::canvas::Canvas::default();
            **latest_painted = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{discovery, identity};

    use super::reset_canvas;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<discovery::Event>();
        app.init_resource::<starboard::canvas::Canvas>();
        app.init_resource::<starboard::input::LatestPaintedCell>();
        app.add_systems(Update, reset_canvas);

        let cell = starboard::canvas::Cell {
            lamport: 3,
            author: 1,
            color: Some(starboard::identity::PaletteColor(1)),
        };
        let mut canvas = app.world_mut().resource_mut::<starboard::canvas::Canvas>();
        let _ = starboard::canvas::write_cell(
            &mut canvas,
            starboard::canvas::CellPos { x: 4, y: 4 },
            cell,
        );
        drop(canvas);
        app.update();
        assert!(
            !app.world()
                .resource::<starboard::canvas::Canvas>()
                .dirty_tiles
                .is_empty()
        );

        app.world_mut()
            .resource_mut::<Messages<discovery::Event>>()
            .write(discovery::Event::Joined(identity::RoomName::from("alpha")));
        app.update();
        assert!(
            app.world()
                .resource::<starboard::canvas::Canvas>()
                .dirty_tiles
                .is_empty()
        );
    }
}
