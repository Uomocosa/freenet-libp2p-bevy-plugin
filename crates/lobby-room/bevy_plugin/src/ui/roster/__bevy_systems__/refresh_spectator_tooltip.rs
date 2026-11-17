use crate::spectator;
use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;

use ui::roster::{SpectatorPill, SpectatorPillTooltip};

pub fn refresh_spectator_tooltip(
    count: Option<Res<spectator::SpectatorCount>>,
    pill: Single<&Hovered, With<SpectatorPill>>,
    tip: Single<(&mut Visibility, &mut Text), With<SpectatorPillTooltip>>,
) {
    let Some(count) = count else {
        return;
    };
    let watching = **count;
    let (mut tip_visibility, mut tip_text) = tip.into_inner();
    let wanted_watching_label = if watching == 1 {
        String::from("1 player watching")
    } else {
        format!("{watching} players watching")
    };
    if **tip_text != wanted_watching_label {
        **tip_text = wanted_watching_label;
    }
    let pill = pill.into_inner();
    let wanted_visibility = if pill.get() {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    if *tip_visibility != wanted_visibility {
        *tip_visibility = wanted_visibility;
    }
}

#[cfg(test)]
mod tests {
    use super::refresh_spectator_tooltip;
    use crate::discovery as core_discovery;
    use crate::spectator;
    use crate::ui;
    use bevy::picking::hover::Hovered;
    use bevy::prelude::*;

    fn app_with() -> (App, Entity, Entity) {
        let mut app = App::new();
        app.insert_resource(spectator::SpectatorCount(4));
        let pill = app
            .world_mut()
            .spawn((ui::roster::SpectatorPill, Hovered::default()))
            .id();
        let tip = app
            .world_mut()
            .spawn((
                ui::roster::SpectatorPillTooltip,
                Visibility::Hidden,
                Text::new(""),
            ))
            .id();
        app.add_systems(Update, refresh_spectator_tooltip);
        (app, pill, tip)
    }

    #[test]
    fn test_usage() {
        let (mut app, _pill, tip) = app_with();
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(tip).expect("tip"),
            Visibility::Hidden
        );
        let Text(watching) = app.world().get::<Text>(tip).expect("tip").clone();
        assert_eq!(watching, String::from("4 players watching"));
    }

    #[test]
    fn one_spectator_is_watching_alone() {
        let (mut app, pill, tip) = app_with();
        **app.world_mut().resource_mut::<spectator::SpectatorCount>() = 1;
        app.world_mut().entity_mut(pill).insert(Hovered(true));
        app.update();
        let Text(watching) = app.world().get::<Text>(tip).expect("tip").clone();
        assert_eq!(watching, String::from("1 player watching"));
    }

    #[test]
    fn hovering_the_pill_shows_the_tooltip() {
        let (mut app, pill, tip) = app_with();
        app.world_mut().entity_mut(pill).insert(Hovered(true));
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(tip).expect("tip"),
            Visibility::Visible
        );
    }

    #[test]
    fn leaving_the_pill_hides_the_tooltip() {
        let (mut app, pill, tip) = app_with();
        app.world_mut().entity_mut(pill).insert(Hovered(true));
        app.update();
        app.world_mut().entity_mut(pill).insert(Hovered(false));
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(tip).expect("tip"),
            Visibility::Hidden
        );
    }

    fn not_hovered(world: &mut World) {
        world.insert_resource(HoverTarget::NotHovered);
    }

    fn hovered(world: &mut World) {
        world.insert_resource(HoverTarget::Hovered);
    }

    #[derive(Resource, Default, Clone, Copy, PartialEq)]
    enum HoverTarget {
        #[default]
        NotHovered,
        Hovered,
    }

    fn keep_hover(
        target: Option<Res<HoverTarget>>,
        pills: Query<Entity, With<ui::roster::SpectatorPill>>,
        mut commands: Commands,
    ) {
        let hover_target = target.map(Res::into_inner);
        let is_hovered = hover_target.is_some_and(|target| *target == HoverTarget::Hovered);
        for pill in &pills {
            commands.entity(pill).insert(Hovered(is_hovered));
        }
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
        app.insert_resource(spectator::SpectatorCount(1));
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
                ui::roster::bevy_systems::refresh_spectator_pill,
                keep_hover,
                refresh_spectator_tooltip,
            )
                .chain(),
        );
    }

    #[test]
    fn refresh_spectator_tooltip_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_spectator_tooltip"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("not hovered"),
                    apply: not_hovered,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("hovered"),
                    apply: hovered,
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
