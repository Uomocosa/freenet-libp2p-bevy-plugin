use crate::spectator;
use crate::ui;
use bevy::prelude::*;

use ui::roster::{SpectatorCountText, SpectatorPill};

pub fn refresh_spectator_pill(
    count: Option<Res<spectator::SpectatorCount>>,
    pill: Single<&mut Visibility, With<SpectatorPill>>,
    text: Single<&mut Text, With<SpectatorCountText>>,
) {
    let Some(count) = count else {
        return;
    };
    let watching = **count;
    let wanted_visibility = if watching > 0 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    let mut pill = pill.into_inner();
    if *pill != wanted_visibility {
        *pill = wanted_visibility;
    }
    let mut text = text.into_inner();
    let wanted_count = watching.to_string();
    if **text != wanted_count {
        **text = wanted_count;
    }
}

#[cfg(test)]
mod tests {
    use super::refresh_spectator_pill;
    use crate::discovery as core_discovery;
    use crate::spectator;
    use crate::ui;
    use bevy::prelude::*;

    fn app_with(watching: usize) -> (App, Entity, Entity) {
        let mut app = App::new();
        app.insert_resource(spectator::SpectatorCount(watching));
        let pill = app
            .world_mut()
            .spawn((ui::roster::SpectatorPill, Visibility::Hidden))
            .id();
        let text = app
            .world_mut()
            .spawn((ui::roster::SpectatorCountText, Text::new("")))
            .id();
        app.add_systems(Update, refresh_spectator_pill);
        (app, pill, text)
    }

    #[test]
    fn test_usage() {
        let (mut app, pill, text) = app_with(3);
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(pill).expect("pill"),
            Visibility::Inherited
        );
        let Text(count) = app.world().get::<Text>(text).expect("count").clone();
        assert_eq!(count, String::from("3"));
    }

    #[test]
    fn nobody_watching_hides_the_pill() {
        let (mut app, pill, text) = app_with(0);
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(pill).expect("pill"),
            Visibility::Hidden
        );
        let Text(count) = app.world().get::<Text>(text).expect("count").clone();
        assert_eq!(count, String::from("0"));
    }

    fn watching_nobody(world: &mut World) {
        world.insert_resource(spectator::SpectatorCount(0));
    }

    fn watching_one(world: &mut World) {
        world.insert_resource(spectator::SpectatorCount(1));
    }

    fn watching_twelve(world: &mut World) {
        world.insert_resource(spectator::SpectatorCount(12));
    }

    fn spawn_mock_screen(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands.spawn((
            ChildOf(root),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(ui::constants::ROOM_COLOR),
            children![(
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(16.0),
                    right: Val::Px(16.0),
                    width: Val::Px(44.0),
                    height: Val::Px(68.0),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.47, 0.29, 0.16)),
                children![Text::new("leave")],
            )],
        ));
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(core_discovery::RoomRoster {
            own: core_discovery::OwnRoomStatus::Connected,
            count: 3,
            max: 8,
            entries: Vec::new(),
        });
        app.insert_resource(spectator::SpectatorCount::default());
        app.add_systems(
            Startup,
            (
                spawn_mock_screen,
                ui::roster::bevy_systems::spawn_roster_bubble,
            ),
        );
        app.add_systems(
            Update,
            (
                ui::roster::bevy_systems::refresh_roster_status,
                refresh_spectator_pill,
                ui::roster::bevy_systems::refresh_spectator_tooltip,
            ),
        );
    }

    #[test]
    fn refresh_spectator_pill_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_spectator_pill"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("no spectators"),
                    apply: watching_nobody,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("one spectator"),
                    apply: watching_one,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("twelve spectators"),
                    apply: watching_twelve,
                },
            ],
            timeline: None,
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
