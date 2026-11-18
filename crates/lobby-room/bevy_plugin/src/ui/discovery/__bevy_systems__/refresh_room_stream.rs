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
mod png_tests {
    use super::refresh_room_stream;
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn refresh_room_stream_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_room_stream"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: |app: &mut App| {
                let row = |name: &str, occupancy: core_discovery::RoomOccupancy| {
                    core_discovery::LobbyRow {
                        name: core_discovery::RoomName::from(name),
                        occupants: 2,
                        room_size: core_discovery::RoomSize::try_from(4).expect("4"),
                        occupancy,
                        friends: 0,
                    }
                };
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
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("alpha streamed"),
                    apply: |world: &mut World| {
                        **world.resource_mut::<ui::discovery::StreamedRoom>() =
                            Some(core_discovery::RoomName::from("alpha"));
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("full room streamed"),
                    apply: |world: &mut World| {
                        **world.resource_mut::<ui::discovery::StreamedRoom>() =
                            Some(core_discovery::RoomName::from("gamma"));
                    },
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
