use std::marker::PhantomData;

use bevy::prelude::*;

use crate::identity;
use crate::peek;
use crate::replica_sync;
use crate::spectator;
use crate::ui;

pub struct RoomPeekPlugin<S: replica_sync::traits::ReplicatedState> {
    pub game: identity::GameId,
    pub marker: PhantomData<S>,
}

impl<S: replica_sync::traits::ReplicatedState> RoomPeekPlugin<S> {
    #[must_use]
    pub const fn new(game: identity::GameId) -> Self {
        Self {
            game,
            marker: PhantomData,
        }
    }
}

#[rustfmt::skip]
impl<S: replica_sync::traits::ReplicatedState> bevy::prelude::Plugin for RoomPeekPlugin<S> {
    fn build(&self, app: &mut App) {
        app.insert_resource(peek::RoomPeekGame(self.game.clone()));
        app.add_message::<spectator::SpectateCommand>();
        app.add_plugins(spectator::SpectatorPlugin::<S>::new(self.game.clone()));
        app.add_systems(
            Update,
            peek::bevy_systems::peek_selected_room.before(replica_sync::ReplicaSyncSet::Drive),
        );
        app.add_systems(
            Update,
            (
                peek::bevy_systems::sync_preview_target::<S>,
                ui::discovery::bevy_systems::show_preview_image::<S>,
            )
                .chain()
                .after(replica_sync::ReplicaSyncSet::Drive),
        );
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::RoomPeekPlugin;
    use crate::identity;
    use crate::peek;
    use crate::replica_sync;
    use crate::spectator;

    #[derive(Default, Resource)]
    struct State;

    impl replica_sync::traits::ReplicatedState for State {
        type Op = u8;
        type Snapshot = u8;

        fn apply(&mut self, _meta: replica_sync::OpMeta, _op: &Self::Op) {}

        fn snapshot(&self) -> Self::Snapshot {
            0
        }

        fn merge(&mut self, _snapshot: Self::Snapshot) {}

        fn slice_content_hashes(&self) -> replica_sync::ReplicaSliceContentHashes {
            Vec::new()
        }

        fn slice_snapshot(&self, _index: replica_sync::ReplicaSliceIndex) -> Self::Snapshot {
            0
        }
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RoomPeekPlugin::<State>::new(identity::GameId::from("game")));
        assert!(app.world().get_resource::<peek::RoomPeekGame>().is_some());
        assert!(
            app.world()
                .get_resource::<Messages<spectator::SpectateCommand>>()
                .is_some()
        );
    }

    const fn preview_noop(_world: &mut World) {}

    fn spawn_mock_screen(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands.spawn((
            ChildOf(root),
            Node {
                width: Val::Percent(70.0),
                height: Val::Percent(50.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.20, 0.42, 0.62)),
        ));
        commands.spawn((
            ChildOf(root),
            Text::new("RoomPeekPlugin"),
            TextFont {
                font_size: FontSize::Px(24.0),
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    }

    fn preview_build(app: &mut App) {
        app.add_plugins(RoomPeekPlugin::<State> {
            game: identity::GameId::from("game"),
            marker: std::marker::PhantomData,
        });
        app.add_systems(Startup, spawn_mock_screen);
    }

    #[test]
    fn room_peek_plugin_ui_scene_preview() {
        lele_bevy_preview::scene::Scene {
            name: String::from("room_peek_plugin"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("lobby"),
                apply: preview_noop,
            }],
            timeline: None,
        }
        .run(&lele_bevy_preview::preview::Config::default())
        .expect("preview");
    }
}
