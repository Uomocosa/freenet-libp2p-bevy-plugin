use bevy::picking::pointer::PointerMap;
use bevy::prelude::*;

use crate::starboard::app::{StarboardPlugin, WallClockSystem};

/// `StarboardPlugin` with every wall-clock system off unless listed in `enabled`.
#[derive(Debug, Default, Clone)]
pub struct StarboardTestPlugin {
    pub enabled: Vec<WallClockSystem>,
}

impl StarboardTestPlugin {
    #[must_use]
    pub fn with(system: WallClockSystem) -> Self {
        Self {
            enabled: vec![system],
        }
    }
}

impl Plugin for StarboardTestPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StarboardPlugin);
        app.init_resource::<PointerMap>();
        for system in WallClockSystem::ALL {
            if !self.enabled.contains(&system) {
                app.configure_sets(Update, system.run_if(|| false));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::input::mouse::AccumulatedMouseScroll;
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;
    use discovery::state_machine::DiscoveryState;
    use freenet_libp2p_bevy_plugin::lobby_room::discovery;
    use freenet_libp2p_bevy_plugin::{identity, time};

    use super::StarboardTestPlugin;
    use crate::starboard;
    use crate::starboard::app::WallClockSystem;

    fn ready_status(me: &identity::PeerId) -> DiscoveryState {
        freenet_libp2p_bevy_plugin::lobby_room::testing::DiscoveryStateBuilder::new(me.clone())
            .in_room(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: time::UnixTime::EPOCH,
                phase: discovery::JoinPhase::Ready,
                members: discovery::Members::new(),
                how_entered: discovery::HowRoomWasEntered::Created,
            })
            .build()
    }

    fn app_with(plugin: StarboardTestPlugin) -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default_nearest(),
        ));
        app.init_resource::<ButtonInput<MouseButton>>();
        app.init_resource::<AccumulatedMouseScroll>();
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            500,
        )));
        app.insert_resource(ready_status(&identity::PeerId::from("me")));
        app.add_plugins(plugin);
        app
    }

    /// The brush the wall clock assigns to `me` right now, and a different brush to plant.
    fn expected_and_planted() -> (
        starboard::identity::DailyBrush,
        starboard::identity::DailyBrush,
    ) {
        let me = identity::PeerId::from("me");
        let status = ready_status(&me);
        let brushes = starboard::identity::room_brushes(
            status.occupants().as_slice(),
            time::UnixTime::EPOCH,
            time::UnixTime::now(),
        );
        let expected = brushes.get(&me).copied().expect("me is an occupant");
        let planted = starboard::identity::DailyBrush {
            shape: starboard::identity::Shape::Star,
            color: starboard::identity::PaletteColor(11),
        };
        if expected == planted {
            let other = starboard::identity::DailyBrush {
                shape: starboard::identity::Shape::Circle,
                color: starboard::identity::PaletteColor(0),
            };
            return (expected, other);
        }
        (expected, planted)
    }

    fn plant(app: &mut App, brush: starboard::identity::DailyBrush) {
        let me = identity::PeerId::from("me");
        app.insert_resource(starboard::input::OwnBrush {
            shape: brush.shape,
            color: brush.color,
            size_index: starboard::canvas::DEFAULT_SIZE_INDEX,
        });
        app.insert_resource(starboard::room::RoomBrushes(
            std::collections::BTreeMap::from([(me, brush)]),
        ));
    }

    /// Two seconds of frames: longer than the 1 s `BrushRefreshTimer`, so it would have fired.
    fn run_past_refresh_interval(app: &mut App) {
        for _ in 0..5 {
            app.update();
        }
    }

    #[test]
    fn test_usage() {
        let app = app_with(StarboardTestPlugin::default());
        assert!(
            app.world()
                .get_resource::<starboard::input::OwnBrush>()
                .is_some()
        );
    }

    #[test]
    fn default_keeps_a_planted_brush_through_the_refresh_interval() {
        let (_expected, planted) = expected_and_planted();
        let mut app = app_with(StarboardTestPlugin::default());
        plant(&mut app, planted);
        run_past_refresh_interval(&mut app);

        let own = *app.world().resource::<starboard::input::OwnBrush>();
        assert_eq!(own.shape, planted.shape);
        assert_eq!(own.color, planted.color);
        let brushes = &app.world().resource::<starboard::room::RoomBrushes>().0;
        assert_eq!(brushes.get(&identity::PeerId::from("me")), Some(&planted));
    }

    #[test]
    fn opting_in_lets_the_refresh_recompute_the_brush() {
        let (expected, planted) = expected_and_planted();
        let mut app = app_with(StarboardTestPlugin::with(WallClockSystem::RoomBrushes));
        plant(&mut app, planted);
        run_past_refresh_interval(&mut app);

        let brushes = &app.world().resource::<starboard::room::RoomBrushes>().0;
        assert_eq!(brushes.get(&identity::PeerId::from("me")), Some(&expected));
        let own = *app.world().resource::<starboard::input::OwnBrush>();
        assert_eq!(own.shape, expected.shape);
        assert_eq!(own.color, expected.color);
    }
}
