use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

use ui::discovery::{
    ROOM_BUBBLE_EMPTY_COLOR, ROOM_BUBBLE_FULL_COLOR, ROOM_BUBBLE_HAS_SPACE_COLOR, RoomStreamName,
    RoomStreamOccupancyBubble, RoomStreamOccupancyText, RoomStreamPanel, StreamDisplay,
    StreamedRoom,
};

type StreamNameText<'w, 's> =
    Single<'w, 's, &'static mut Text, (With<RoomStreamName>, Without<RoomStreamOccupancyText>)>;
type StreamOccupancyText<'w, 's> =
    Single<'w, 's, &'static mut Text, (With<RoomStreamOccupancyText>, Without<RoomStreamName>)>;

pub fn refresh_room_stream(
    streamed: Res<StreamedRoom>,
    rows: Option<Res<core_discovery::LobbyRows>>,
    mut display: ResMut<StreamDisplay>,
    panel: Option<Single<&mut Visibility, With<RoomStreamPanel>>>,
    name: Option<StreamNameText>,
    occupancy_bubble: Option<Single<&mut BackgroundColor, With<RoomStreamOccupancyBubble>>>,
    occupancy_text: Option<StreamOccupancyText>,
) {
    let Some(panel) = panel else {
        return;
    };
    let streamed = streamed.into_inner();
    let rows = rows.map(Res::into_inner);
    let row = (**streamed)
        .as_ref()
        .and_then(|name| rows?.iter().find(|row| row.name == *name));
    let Some(row) = row else {
        let mut panel = panel.into_inner();
        if *panel != Visibility::Hidden {
            *panel = Visibility::Hidden;
        }
        display.show_panel();
        return;
    };
    let mut panel = panel.into_inner();
    if *panel != Visibility::Visible {
        *panel = Visibility::Visible;
    }
    if let Some(name) = name {
        let mut text = name.into_inner();
        if text.as_str() != row.name.as_str() {
            *text = Text::new(row.name.as_str());
        }
    }
    if let Some(occupancy_text) = occupancy_text {
        let wanted = format!("{}/{}", row.occupants, row.room_size);
        let mut text = occupancy_text.into_inner();
        if text.as_str() != wanted.as_str() {
            *text = Text::new(wanted);
        }
    }
    if let Some(occupancy_bubble) = occupancy_bubble {
        let wanted = match row.occupancy {
            core_discovery::RoomOccupancy::HasSpace => ROOM_BUBBLE_HAS_SPACE_COLOR,
            core_discovery::RoomOccupancy::Empty => ROOM_BUBBLE_EMPTY_COLOR,
            core_discovery::RoomOccupancy::Full => ROOM_BUBBLE_FULL_COLOR,
        };
        let BackgroundColor(existing) = &mut *occupancy_bubble.into_inner();
        if *existing != wanted {
            *existing = wanted;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    use super::refresh_room_stream;

    fn row(name: &str, occupancy: core_discovery::RoomOccupancy) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from(name),
            occupants: 2,
            room_size: core_discovery::RoomSize::try_from(8).expect("8"),
            occupancy,
            friends: 0,
        }
    }

    fn app_with(streamed: Option<&str>, rows: Vec<core_discovery::LobbyRow>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ui::discovery::StreamDisplay>();
        app.insert_resource(ui::discovery::StreamedRoom(
            streamed.map(core_discovery::RoomName::from),
        ));
        app.insert_resource(core_discovery::LobbyRows(rows));
        app.add_systems(Startup, ui::discovery::bevy_systems::spawn_room_stream);
        app.update();
        app.add_systems(Update, refresh_room_stream);
        app
    }

    fn panel_visibility(app: &mut App) -> Visibility {
        let mut panels = app
            .world_mut()
            .query_filtered::<&Visibility, With<ui::discovery::RoomStreamPanel>>();
        *panels.single(app.world()).expect("stream panel")
    }

    fn name_text(app: &mut App) -> String {
        let mut names = app
            .world_mut()
            .query_filtered::<&Text, With<ui::discovery::RoomStreamName>>();
        names
            .single(app.world())
            .expect("stream name")
            .as_str()
            .to_string()
    }

    fn occupancy_text(app: &mut App) -> String {
        let mut texts = app
            .world_mut()
            .query_filtered::<&Text, With<ui::discovery::RoomStreamOccupancyText>>();
        texts
            .single(app.world())
            .expect("occupancy text")
            .as_str()
            .to_string()
    }

    fn occupancy_color(app: &mut App) -> Color {
        let mut bubbles = app
            .world_mut()
            .query_filtered::<&BackgroundColor, With<ui::discovery::RoomStreamOccupancyBubble>>();
        let BackgroundColor(color) = *bubbles.single(app.world()).expect("occupancy bubble");
        color
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(
            Some("alpha"),
            vec![row("alpha", core_discovery::RoomOccupancy::HasSpace)],
        );
        app.update();
        assert_eq!(panel_visibility(&mut app), Visibility::Visible);
        assert_eq!(name_text(&mut app), "alpha");
        assert_eq!(occupancy_text(&mut app), "2/8");
        assert_eq!(
            occupancy_color(&mut app),
            ui::discovery::ROOM_BUBBLE_HAS_SPACE_COLOR
        );
    }

    #[test]
    fn without_a_streamed_room_the_panel_hides_and_returns_to_the_panel_display() {
        let mut app = app_with(
            None,
            vec![row("alpha", core_discovery::RoomOccupancy::HasSpace)],
        );
        *app.world_mut()
            .resource_mut::<ui::discovery::StreamDisplay>() =
            ui::discovery::StreamDisplay::Fullscreen;
        app.update();
        assert_eq!(panel_visibility(&mut app), Visibility::Hidden);
        assert_eq!(
            *app.world().resource::<ui::discovery::StreamDisplay>(),
            ui::discovery::StreamDisplay::Panel
        );
    }

    #[test]
    fn a_streamed_room_missing_from_the_rows_hides_the_panel() {
        let mut app = app_with(
            Some("alpha"),
            vec![row("beta", core_discovery::RoomOccupancy::HasSpace)],
        );
        app.update();
        assert_eq!(panel_visibility(&mut app), Visibility::Hidden);
    }

    #[test]
    fn a_full_room_shows_the_full_bubble() {
        let mut app = app_with(
            Some("gamma"),
            vec![row("gamma", core_discovery::RoomOccupancy::Full)],
        );
        app.update();
        assert_eq!(name_text(&mut app), "gamma");
        assert_eq!(
            occupancy_color(&mut app),
            ui::discovery::ROOM_BUBBLE_FULL_COLOR
        );
    }
}

#[cfg(test)]
mod png_tests {
    use super::refresh_room_stream;
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    fn row(name: &str, occupancy: core_discovery::RoomOccupancy) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from(name),
            occupants: 2,
            room_size: core_discovery::RoomSize::try_from(4).expect("4"),
            occupancy,
            friends: 0,
        }
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(ui::discovery::StreamDisplay::default());
        app.insert_resource(ui::discovery::StreamedRoom(Some(
            core_discovery::RoomName::from("alpha"),
        )));
        app.insert_resource(core_discovery::LobbyRows(vec![
            row("alpha", core_discovery::RoomOccupancy::HasSpace),
            row("gamma", core_discovery::RoomOccupancy::Full),
        ]));
        app.add_systems(Startup, ui::discovery::bevy_systems::spawn_room_stream);
        app.add_systems(
            Update,
            (
                refresh_room_stream,
                ui::discovery::bevy_systems::layout_stream_panel,
            )
                .chain(),
        );
    }

    fn stream_alpha(world: &mut World) {
        **world.resource_mut::<ui::discovery::StreamedRoom>() =
            Some(core_discovery::RoomName::from("alpha"));
    }

    fn stream_gamma(world: &mut World) {
        **world.resource_mut::<ui::discovery::StreamedRoom>() =
            Some(core_discovery::RoomName::from("gamma"));
    }

    #[test]
    fn refresh_room_stream_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_room_stream"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("alpha streamed"),
                    apply: stream_alpha,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("full room streamed"),
                    apply: stream_gamma,
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
