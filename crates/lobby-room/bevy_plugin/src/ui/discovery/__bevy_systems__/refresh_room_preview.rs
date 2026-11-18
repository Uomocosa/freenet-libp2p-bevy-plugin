use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

use ui::discovery::{RoomPreviewPanel, SelectedRoom, StreamedRoom};

pub fn refresh_room_preview(
    selection: Res<SelectedRoom>,
    streamed: Res<StreamedRoom>,
    rows: Option<Res<core_discovery::LobbyRows>>,
    panel: Option<Single<&mut Visibility, With<RoomPreviewPanel>>>,
) {
    let Some(panel) = panel else {
        return;
    };
    let selection = selection.into_inner();
    let streamed = streamed.into_inner();
    let rows = rows.map(Res::into_inner);
    let is_previewed = (**selection)
        .as_ref()
        .filter(|name| (**streamed).as_ref() != Some(*name))
        .is_some_and(|name| rows.is_some_and(|rows| rows.iter().any(|row| row.name == *name)));
    let wanted = if is_previewed {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    let mut panel = panel.into_inner();
    if *panel != wanted {
        *panel = wanted;
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    use super::refresh_room_preview;

    #[test]
    fn refresh_room_preview_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_room_preview"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: |app: &mut App| {
                let row = |name: &str, occupants: usize| core_discovery::LobbyRow {
                    name: core_discovery::RoomName::from(name),
                    occupants,
                    room_size: core_discovery::RoomSize::try_from(8).expect("8"),
                    occupancy: core_discovery::RoomOccupancy::HasSpace,
                    friends: 0,
                };
                app.insert_resource(core_discovery::LobbyRows(vec![
                    row("alpha", 2),
                    row("gamma", 4),
                ]));
                app.insert_resource(ui::discovery::StreamedRoom::default());
                app.insert_resource(ui::discovery::SelectedRoom(Some(
                    core_discovery::RoomName::from("alpha"),
                )));
                app.add_systems(Startup, ui::discovery::bevy_systems::spawn_room_preview);
                app.add_systems(Update, refresh_room_preview);
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("alpha hovered"),
                    apply: |world: &mut World| {
                        **world.resource_mut::<ui::discovery::SelectedRoom>() =
                            Some(core_discovery::RoomName::from("alpha"));
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("gamma hovered"),
                    apply: |world: &mut World| {
                        **world.resource_mut::<ui::discovery::SelectedRoom>() =
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
