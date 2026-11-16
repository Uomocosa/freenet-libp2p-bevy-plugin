use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

use ui::discovery::{
    ROOM_BUBBLE_EMPTY_COLOR, ROOM_BUBBLE_FULL_COLOR, ROOM_BUBBLE_HAS_SPACE_COLOR,
    RoomPreviewOccupancyBubble, RoomPreviewOccupancyText, RoomPreviewPanel, RoomPreviewTitle,
    SelectedRoom,
};

type PreviewTitleText<'w, 's> =
    Single<'w, 's, &'static mut Text, (With<RoomPreviewTitle>, Without<RoomPreviewOccupancyText>)>;
type PreviewOccupancyText<'w, 's> =
    Single<'w, 's, &'static mut Text, (With<RoomPreviewOccupancyText>, Without<RoomPreviewTitle>)>;

pub fn refresh_room_preview(
    selection: Res<SelectedRoom>,
    rows: Option<Res<core_discovery::LobbyRows>>,
    panel: Option<Single<&mut Visibility, With<RoomPreviewPanel>>>,
    title: Option<PreviewTitleText>,
    occupancy_bubble: Option<Single<&mut BackgroundColor, With<RoomPreviewOccupancyBubble>>>,
    occupancy_text: Option<PreviewOccupancyText>,
) {
    let Some(panel) = panel else {
        return;
    };
    let Some(title) = title else {
        return;
    };
    let Some(occupancy_bubble) = occupancy_bubble else {
        return;
    };
    let Some(occupancy_text) = occupancy_text else {
        return;
    };
    let selection = selection.into_inner();
    let rows = rows.map(Res::into_inner);
    let row = selection
        .room
        .as_ref()
        .and_then(|name| rows?.iter().find(|row| row.name == *name));
    let Some(row) = row else {
        *panel.into_inner() = Visibility::Hidden;
        return;
    };
    *panel.into_inner() = Visibility::Visible;
    let mut title = title.into_inner();
    if title.as_str() != row.name.as_str() {
        *title = Text::new(row.name.as_str());
    }
    let occupancy = format!("{}/{}", row.occupants, row.room_size);
    let mut occupancy_text = occupancy_text.into_inner();
    if occupancy_text.as_str() != occupancy.as_str() {
        *occupancy_text = Text::new(occupancy);
    }
    let color = match row.occupancy {
        core_discovery::RoomOccupancy::HasSpace => ROOM_BUBBLE_HAS_SPACE_COLOR,
        core_discovery::RoomOccupancy::Empty => ROOM_BUBBLE_EMPTY_COLOR,
        core_discovery::RoomOccupancy::Full => ROOM_BUBBLE_FULL_COLOR,
    };
    let BackgroundColor(existing) = &mut *occupancy_bubble.into_inner();
    if *existing != color {
        *existing = color;
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    use super::refresh_room_preview;

    fn row(name: &str, occupants: usize) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from(name),
            occupants,
            room_size: core_discovery::RoomSize::try_from(8).expect("8"),
            occupancy: core_discovery::RoomOccupancy::HasSpace,
            friends: 0,
        }
    }

    fn app_with(
        selection: ui::discovery::SelectedRoom,
        rows: Vec<core_discovery::LobbyRow>,
    ) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(selection);
        app.insert_resource(core_discovery::LobbyRows(rows));
        app.add_systems(Startup, ui::discovery::bevy_systems::spawn_room_preview);
        app.update();
        app.add_systems(Update, refresh_room_preview);
        app
    }

    fn panel_visibility(app: &mut App) -> Visibility {
        let mut panels = app
            .world_mut()
            .query_filtered::<&Visibility, With<ui::discovery::RoomPreviewPanel>>();
        *panels.single(app.world()).expect("preview panel")
    }

    fn title_text(app: &mut App) -> String {
        let mut titles = app
            .world_mut()
            .query_filtered::<&Text, With<ui::discovery::RoomPreviewTitle>>();
        titles
            .single(app.world())
            .expect("preview title")
            .as_str()
            .to_string()
    }

    fn occupancy_text(app: &mut App) -> String {
        let mut texts = app
            .world_mut()
            .query_filtered::<&Text, With<ui::discovery::RoomPreviewOccupancyText>>();
        texts
            .single(app.world())
            .expect("occupancy text")
            .as_str()
            .to_string()
    }

    fn occupancy_color(app: &mut App) -> Color {
        let mut bubbles = app
            .world_mut()
            .query_filtered::<&BackgroundColor, With<ui::discovery::RoomPreviewOccupancyBubble>>();
        let BackgroundColor(color) = *bubbles.single(app.world()).expect("occupancy bubble");
        color
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(
            ui::discovery::SelectedRoom {
                room: Some(core_discovery::RoomName::from("alpha")),
                is_locked: true,
            },
            vec![row("alpha", 2)],
        );
        app.update();
        assert_eq!(panel_visibility(&mut app), Visibility::Visible);
        assert_eq!(title_text(&mut app), "alpha");
        assert_eq!(occupancy_text(&mut app), "2/8");
        assert_eq!(
            occupancy_color(&mut app),
            ui::discovery::ROOM_BUBBLE_HAS_SPACE_COLOR
        );
    }

    #[test]
    fn an_empty_selection_hides_the_panel() {
        let mut app = app_with(
            ui::discovery::SelectedRoom::default(),
            vec![row("alpha", 2)],
        );
        app.update();
        assert_eq!(panel_visibility(&mut app), Visibility::Hidden);
    }

    #[test]
    fn a_selection_missing_from_the_rows_hides_the_panel() {
        let mut app = app_with(
            ui::discovery::SelectedRoom {
                room: Some(core_discovery::RoomName::from("alpha")),
                is_locked: true,
            },
            vec![row("beta", 0)],
        );
        app.update();
        assert_eq!(panel_visibility(&mut app), Visibility::Hidden);
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(core_discovery::LobbyRows(vec![
            row("alpha", 2),
            row("gamma", 4),
        ]));
        app.insert_resource(ui::discovery::SelectedRoom {
            room: Some(core_discovery::RoomName::from("alpha")),
            is_locked: true,
        });
        app.add_systems(Startup, ui::discovery::bevy_systems::spawn_room_preview);
        app.add_systems(Update, refresh_room_preview);
    }

    fn select_room(world: &mut World, room: &str) {
        let mut selection = world.resource_mut::<ui::discovery::SelectedRoom>();
        selection.room = Some(core_discovery::RoomName::from(room));
        selection.is_locked = true;
    }

    fn select_alpha(world: &mut World) {
        select_room(world, "alpha");
    }

    fn select_gamma(world: &mut World) {
        select_room(world, "gamma");
    }

    #[test]
    fn refresh_room_preview_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_room_preview"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("alpha selected"),
                    apply: select_alpha,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("gamma selected"),
                    apply: select_gamma,
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
