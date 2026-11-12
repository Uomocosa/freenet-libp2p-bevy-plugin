use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::discovery;

use crate::starboard;

pub fn reset_canvas(
    mut joins: MessageReader<discovery::Event>,
    mut canvas: ResMut<starboard::canvas::Canvas>,
    mut latest_painted: ResMut<starboard::input::LatestPaintedCell>,
    mut pending: ResMut<starboard::input::PendingPressState>,
) {
    for event in joins.read() {
        if matches!(event, discovery::Event::Joined(_)) {
            *canvas = starboard::canvas::Canvas::default();
            *latest_painted = starboard::input::LatestPaintedCell::default();
            **pending = None;
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
        app.init_resource::<starboard::input::PendingPressState>();
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

    #[test]
    fn a_room_change_clears_the_pending_press() {
        let mut app = App::new();
        app.add_message::<discovery::Event>();
        app.init_resource::<starboard::canvas::Canvas>();
        app.init_resource::<starboard::input::LatestPaintedCell>();
        app.init_resource::<starboard::input::PendingPressState>();
        app.add_systems(Update, reset_canvas);

        let press_cell = starboard::canvas::SubCellPos { x: 76, y: 76 };
        let press = starboard::input::PendingPress::begin(
            MouseButton::Left,
            press_cell,
            Vec2::ZERO,
            0.0,
            5,
        );
        **app
            .world_mut()
            .resource_mut::<starboard::input::PendingPressState>() = Some(press);
        app.world_mut()
            .resource_mut::<Messages<discovery::Event>>()
            .write(discovery::Event::Joined(identity::RoomName::from("beta")));
        app.update();

        assert!(
            app.world()
                .resource::<starboard::input::PendingPressState>()
                .is_none()
        );
    }
}
