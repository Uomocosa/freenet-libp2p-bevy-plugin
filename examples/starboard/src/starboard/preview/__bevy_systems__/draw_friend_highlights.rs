use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::lobby_room::{discovery, ui};

use crate::starboard;

pub fn draw_friend_highlights(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    peeked: Option<Res<ui::discovery::PeekedRoomFriends>>,
    cursors: Res<starboard::preview::SlotCursors>,
    mut highlights: Query<(Entity, &starboard::preview::FriendHighlight, &mut Transform)>,
) {
    let config = starboard::preview::PREVIEW_SLOT;
    let view = starboard::render::CanvasView::fit(Vec2::splat(config.board_side));
    let window = Vec2::splat(config.board_side);
    let layers = RenderLayers::layer(config.layer);
    let peeked = peeked.map(Res::into_inner);
    let friends: &[discovery::RoomFriend] = peeked.map_or(&[], |peeked| &**peeked);
    let slot_cursors = cursors.into_inner().get(&config.slot);
    for (entity, highlight, mut transform) in &mut highlights {
        let is_peeked = friends.iter().any(|friend| friend.peer == **highlight);
        let Some(cursor) = slot_cursors.and_then(|cursors| cursors.get(&**highlight)) else {
            commands.entity(entity).despawn();
            continue;
        };
        if !is_peeked {
            commands.entity(entity).despawn();
            continue;
        }
        let at = starboard::render::cell_center(cursor.cell, &view, window);
        transform.translation = Vec3::new(at.x, at.y, starboard::preview::FRIEND_HIGHLIGHT_Z);
    }
    for friend in friends {
        let Some(cursor) = slot_cursors.and_then(|cursors| cursors.get(&friend.peer)) else {
            continue;
        };
        let is_already_drawn = highlights
            .iter()
            .any(|(_, highlight, _)| **highlight == friend.peer);
        if is_already_drawn {
            continue;
        }
        let at = starboard::render::cell_center(cursor.cell, &view, window);
        let _ = starboard::preview::bevy_systems::spawn_friend_highlight(
            &mut commands,
            &mut images,
            friend,
            at,
            &layers,
        );
    }
}

#[cfg(test)]
mod tests {
    use bevy::camera::visibility::RenderLayers;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::identity;
    use freenet_libp2p_bevy_plugin::lobby_room::{discovery, spectator, ui};

    use super::draw_friend_highlights;
    use crate::starboard;

    fn friend(peer: &str, name: &str) -> discovery::RoomFriend {
        discovery::RoomFriend {
            peer: identity::PeerId::from(peer),
            name: identity::Username::from(name),
        }
    }

    fn cursor(x: u16) -> starboard::room::CursorUpdate {
        starboard::room::CursorUpdate {
            cell: starboard::canvas::CellPos { x, y: 300 },
            size_index: starboard::canvas::DEFAULT_SIZE_INDEX,
        }
    }

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default_nearest(),
        ));
        app.init_resource::<starboard::preview::SlotCursors>()
            .init_resource::<ui::discovery::PeekedRoomFriends>()
            .add_systems(Update, draw_friend_highlights);
        app
    }

    fn peek(app: &mut App, friends: Vec<discovery::RoomFriend>) {
        *app.world_mut()
            .resource_mut::<ui::discovery::PeekedRoomFriends>() =
            ui::discovery::PeekedRoomFriends(friends);
    }

    fn show_cursor(app: &mut App, slot: spectator::SpectateSlot, peer: &str, x: u16) {
        app.world_mut()
            .resource_mut::<starboard::preview::SlotCursors>()
            .entry(slot)
            .or_default()
            .insert(identity::PeerId::from(peer), cursor(x));
    }

    fn highlight_count(app: &mut App) -> usize {
        let world = app.world_mut();
        let mut highlights =
            world.query_filtered::<(), With<starboard::preview::FriendHighlight>>();
        highlights.iter(world).count()
    }

    #[test]
    fn test_usage() {
        let mut app = app();
        peek(
            &mut app,
            vec![friend("peer-a", "alice"), friend("peer-b", "bob")],
        );
        show_cursor(&mut app, spectator::SpectateSlot::Preview, "peer-a", 300);
        app.update();

        assert_eq!(highlight_count(&mut app), 1);
    }

    #[test]
    fn a_friend_with_no_cursor_on_the_preview_board_gets_no_highlight() {
        let mut app = app();
        peek(
            &mut app,
            vec![friend("peer-a", "alice"), friend("peer-b", "bob")],
        );
        show_cursor(&mut app, spectator::SpectateSlot::Preview, "peer-a", 300);
        show_cursor(&mut app, spectator::SpectateSlot::Stream, "peer-b", 400);
        app.update();

        assert_eq!(highlight_count(&mut app), 1);
    }

    #[test]
    fn clearing_the_peeked_friends_despawns_their_highlights() {
        let mut app = app();
        peek(&mut app, vec![friend("peer-a", "alice")]);
        show_cursor(&mut app, spectator::SpectateSlot::Preview, "peer-a", 300);
        app.update();
        assert_eq!(highlight_count(&mut app), 1);

        peek(&mut app, Vec::new());
        app.update();
        assert_eq!(highlight_count(&mut app), 0);
    }

    fn preview_build(app: &mut App) {
        app.init_resource::<starboard::preview::SlotCursors>()
            .init_resource::<ui::discovery::PeekedRoomFriends>()
            .add_systems(Startup, show_two_friends);
    }

    fn show_two_friends(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
        let side = starboard::preview::PREVIEW_BOARD_SIDE;
        let view = starboard::render::CanvasView::fit(Vec2::splat(side));
        let window = Vec2::splat(side);
        let layers = RenderLayers::default();
        let board = images.add(starboard::render::CanvasView::checkered_image());
        commands.spawn((
            Sprite {
                image: board,
                custom_size: Some(Vec2::splat(side)),
                ..default()
            },
            layers.clone(),
        ));
        let friends = [
            (
                friend("peer-a", "alice"),
                starboard::canvas::CellPos { x: 300, y: 300 },
            ),
            (
                friend("peer-b", "bob"),
                starboard::canvas::CellPos { x: 700, y: 650 },
            ),
        ];
        for (friend, cell) in friends {
            let at = starboard::render::cell_center(cell, &view, window);
            let _ = starboard::preview::bevy_systems::spawn_friend_highlight(
                &mut commands,
                &mut images,
                &friend,
                at,
                &layers,
            );
        }
    }

    const fn preview_noop(_world: &mut World) {}

    fn config() -> lele_bevy_preview::preview::Config {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        }
    }

    fn scene() -> lele_bevy_preview::scene::Scene {
        lele_bevy_preview::scene::Scene {
            name: String::from("draw_friend_highlights"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("two friends highlighted"),
                apply: preview_noop,
            }],
            timeline: None,
        }
    }

    #[test]
    fn draw_friend_highlights_ui_png_preview() {
        scene().run(&config()).expect("preview");
    }
}
