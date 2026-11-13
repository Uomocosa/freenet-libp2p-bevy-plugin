use bevy::picking::hover::Hovered;
use bevy::prelude::*;

use crate::roster;
use roster::{RosterCount, RosterCountTooltip, RosterDot, RosterDotTooltip};

pub fn refresh_roster_tooltip(
    dots: Query<&Hovered, With<RosterDot>>,
    counts: Query<&Hovered, With<RosterCount>>,
    dot_tip: Single<&mut Visibility, (With<RosterDotTooltip>, Without<RosterCountTooltip>)>,
    count_tip: Single<&mut Visibility, (With<RosterCountTooltip>, Without<RosterDotTooltip>)>,
) {
    let is_dot_hovered = dots.iter().any(Hovered::get);
    let is_count_hovered = counts.iter().any(Hovered::get);
    let wanted_dot_tip = if is_dot_hovered {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    let mut dot_tip = dot_tip.into_inner();
    if *dot_tip != wanted_dot_tip {
        *dot_tip = wanted_dot_tip;
    }
    let wanted_count_tip = if is_count_hovered {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    let mut count_tip = count_tip.into_inner();
    if *count_tip != wanted_count_tip {
        *count_tip = wanted_count_tip;
    }
}

#[cfg(test)]
mod tests {
    use super::refresh_roster_tooltip;
    use crate::roster;
    use bevy::picking::hover::Hovered;

    use crate::basic::constants::ROOM_COLOR;
    use bevy::prelude::*;

    fn app_with() -> (App, Entity, Entity, Entity, Entity) {
        let mut app = App::new();
        let dot_tip = app
            .world_mut()
            .spawn((roster::RosterDotTooltip, Visibility::Hidden))
            .id();
        let count_tip = app
            .world_mut()
            .spawn((roster::RosterCountTooltip, Visibility::Hidden))
            .id();
        let dot = app
            .world_mut()
            .spawn((roster::RosterDot, Hovered::default()))
            .id();
        let count = app
            .world_mut()
            .spawn((roster::RosterCount, Hovered::default()))
            .id();
        app.add_systems(Update, refresh_roster_tooltip);
        (app, dot_tip, count_tip, dot, count)
    }

    #[test]
    fn test_usage() {
        let (mut app, _dot_tip, _count_tip, _dot, _count) = app_with();
        app.update();
    }

    #[test]
    fn hovering_the_dot_shows_the_status_word() {
        let (mut app, dot_tip, count_tip, dot, _count) = app_with();
        app.world_mut().entity_mut(dot).insert(Hovered(true));
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(dot_tip).expect("dot tip"),
            Visibility::Visible
        );
        assert_eq!(
            *app.world().get::<Visibility>(count_tip).expect("count tip"),
            Visibility::Hidden
        );
    }

    #[test]
    fn hovering_the_count_shows_the_peers_in_the_room() {
        let (mut app, dot_tip, count_tip, _dot, count) = app_with();
        app.world_mut().entity_mut(count).insert(Hovered(true));
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(dot_tip).expect("dot tip"),
            Visibility::Hidden
        );
        assert_eq!(
            *app.world().get::<Visibility>(count_tip).expect("count tip"),
            Visibility::Visible
        );
    }

    #[test]
    fn hovering_nothing_hides_both_tooltips() {
        let (mut app, dot_tip, count_tip, dot, _count) = app_with();
        app.world_mut().entity_mut(dot).insert(Hovered(true));
        app.update();
        app.world_mut().entity_mut(dot).insert(Hovered(false));
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(dot_tip).expect("dot tip"),
            Visibility::Hidden
        );
        assert_eq!(
            *app.world().get::<Visibility>(count_tip).expect("count tip"),
            Visibility::Hidden
        );
    }

    fn hover_dot(world: &mut World) {
        world.insert_resource(HoverTarget::Dot);
    }

    fn hover_count(world: &mut World) {
        world.insert_resource(HoverTarget::Count);
    }

    fn hover_nothing(world: &mut World) {
        world.insert_resource(HoverTarget::Nothing);
    }

    #[derive(Resource, Default, Clone, Copy, PartialEq)]
    enum HoverTarget {
        #[default]
        Nothing,
        Dot,
        Count,
    }

    fn keep_hover(
        target: Option<Res<HoverTarget>>,
        dots: Query<Entity, With<roster::RosterDot>>,
        counts: Query<Entity, With<roster::RosterCount>>,
        mut commands: Commands,
    ) {
        let hover_target = target.map(Res::into_inner);
        let is_dot_hovered = hover_target.is_some_and(|target| *target == HoverTarget::Dot);
        let is_count_hovered = hover_target.is_some_and(|target| *target == HoverTarget::Count);
        for entity in &dots {
            commands.entity(entity).insert(Hovered(is_dot_hovered));
        }
        for entity in &counts {
            commands.entity(entity).insert(Hovered(is_count_hovered));
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
            BackgroundColor(ROOM_COLOR),
        ));
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(freenet_libp2p_bevy_lobby_room::discovery::RoomRoster {
            own: freenet_libp2p_bevy_lobby_room::discovery::OwnRoomStatus::Connected,
            count: 3,
            max: 8,
            entries: vec![
                freenet_libp2p_bevy_lobby_room::discovery::RoomRosterEntry {
                    id: freenet_libp2p_bevy_core::identity::PeerId::from("12D3KooWcarol0Zm9c"),
                    name: String::from("carol"),
                    status: freenet_libp2p_bevy_lobby_room::discovery::MemberRoomStatus::Connecting,
                    is_me: false,
                },
                freenet_libp2p_bevy_lobby_room::discovery::RoomRosterEntry {
                    id: freenet_libp2p_bevy_core::identity::PeerId::from("12D3KooWbob000Kd2a"),
                    name: String::from("bob"),
                    status: freenet_libp2p_bevy_lobby_room::discovery::MemberRoomStatus::Connected,
                    is_me: false,
                },
                freenet_libp2p_bevy_lobby_room::discovery::RoomRosterEntry {
                    id: freenet_libp2p_bevy_core::identity::PeerId::from("me"),
                    name: String::from("alice"),
                    status: freenet_libp2p_bevy_lobby_room::discovery::MemberRoomStatus::Connected,
                    is_me: true,
                },
            ],
        });
        app.add_systems(
            Startup,
            (spawn_mock_screen, roster::bevy_systems::spawn_roster_bubble),
        );
        app.add_systems(
            Update,
            (
                keep_hover,
                roster::bevy_systems::refresh_roster_status,
                roster::bevy_systems::refresh_roster_tooltips_text,
                refresh_roster_tooltip,
            )
                .chain(),
        );
    }

    #[test]
    fn refresh_roster_tooltip_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_roster_tooltip"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("dot hovered"),
                    apply: hover_dot,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("count hovered"),
                    apply: hover_count,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("not hovered"),
                    apply: hover_nothing,
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
