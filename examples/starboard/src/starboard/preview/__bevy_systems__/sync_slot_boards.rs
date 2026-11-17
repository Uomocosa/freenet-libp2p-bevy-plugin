use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::lobby_room::peek;

use crate::starboard;

pub fn sync_slot_boards(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    preview_target: Option<Res<peek::RoomPreviewTarget>>,
    stream_target: Option<Res<peek::RoomStreamTarget>>,
    boards: Query<(Entity, &starboard::preview::SlotBoardSprite)>,
) {
    let [preview, stream] = starboard::preview::SLOT_RENDERS;
    let targets = [
        (preview, preview_target.map(|target| target.image.clone())),
        (stream, stream_target.map(|target| target.image.clone())),
    ];
    for (config, image) in targets {
        let has_target = image.is_some();
        let existing = boards
            .iter()
            .find(|(_, board)| ***board == config.slot)
            .map(|(entity, _)| entity);
        match (has_target, existing) {
            (false, Some(entity)) => {
                commands.entity(entity).despawn();
            }
            (true, None) => {
                let image = images.add(starboard::render::CanvasView::checkered_image());
                commands.spawn((
                    starboard::preview::SlotBoardSprite(config.slot),
                    Sprite {
                        image,
                        custom_size: Some(Vec2::splat(config.board_side)),
                        ..default()
                    },
                    RenderLayers::layer(config.layer),
                ));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::camera::visibility::RenderLayers;
    use bevy::prelude::*;
    use discovery::state_machine::DiscoveryState;
    use freenet_libp2p_bevy_plugin::identity;
    use freenet_libp2p_bevy_plugin::lobby_room::{discovery, peek, spectator};
    use freenet_libp2p_bevy_plugin::time;

    use super::sync_slot_boards;
    use crate::starboard;

    fn target_image(app: &mut App, width: u32, height: u32) -> Handle<Image> {
        app.world_mut()
            .resource_mut::<Assets<Image>>()
            .add(Image::new_target_texture(
                width,
                height,
                bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb,
                None,
            ))
    }

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default_nearest(),
        ));
        app.add_systems(Update, sync_slot_boards);
        app
    }

    fn board(
        app: &mut App,
        slot: spectator::SpectateSlot,
    ) -> Option<(Handle<Image>, Vec2, RenderLayers)> {
        let world = app.world_mut();
        let mut boards =
            world.query::<(&starboard::preview::SlotBoardSprite, &Sprite, &RenderLayers)>();
        boards
            .iter(world)
            .find(|(board, ..)| ***board == slot)
            .map(|(_, sprite, layers)| {
                (
                    sprite.image.clone(),
                    sprite.custom_size.unwrap_or(Vec2::ZERO),
                    layers.clone(),
                )
            })
    }

    fn start_both(app: &mut App) {
        let preview = target_image(app, 320, 200);
        let stream = target_image(app, 1280, 800);
        app.insert_resource(peek::RoomPreviewTarget {
            zone: identity::ZoneName::from("alpha"),
            image: preview,
        });
        app.insert_resource(peek::RoomStreamTarget {
            zone: identity::ZoneName::from("beta"),
            image: stream,
        });
    }

    #[test]
    fn test_usage() {
        let mut app = app();
        start_both(&mut app);
        app.update();

        let (_, size, layers) =
            board(&mut app, spectator::SpectateSlot::Preview).expect("a preview board");
        assert_eq!(size, Vec2::splat(starboard::preview::PREVIEW_BOARD_SIDE));
        assert_eq!(
            layers,
            RenderLayers::layer(starboard::preview::PREVIEW_LAYER)
        );
    }

    #[test]
    fn each_slot_board_sits_on_its_own_layer_at_its_own_size() {
        let mut app = app();
        start_both(&mut app);
        app.update();

        let (_, preview_size, preview_layers) =
            board(&mut app, spectator::SpectateSlot::Preview).expect("a preview board");
        assert_eq!(
            preview_size,
            Vec2::splat(starboard::preview::PREVIEW_BOARD_SIDE)
        );
        assert_eq!(
            preview_layers,
            RenderLayers::layer(starboard::preview::PREVIEW_LAYER)
        );

        let (_, stream_size, stream_layers) =
            board(&mut app, spectator::SpectateSlot::Stream).expect("a stream board");
        assert_eq!(
            stream_size,
            Vec2::splat(starboard::preview::STREAM_BOARD_SIDE)
        );
        assert_eq!(
            stream_layers,
            RenderLayers::layer(starboard::preview::STREAM_LAYER)
        );
    }

    #[test]
    fn dropping_a_target_despawns_only_its_board() {
        let mut app = app();
        start_both(&mut app);
        app.update();
        assert!(board(&mut app, spectator::SpectateSlot::Preview).is_some());
        assert!(board(&mut app, spectator::SpectateSlot::Stream).is_some());

        app.world_mut().remove_resource::<peek::RoomPreviewTarget>();
        app.update();
        assert!(board(&mut app, spectator::SpectateSlot::Preview).is_none());
        assert!(board(&mut app, spectator::SpectateSlot::Stream).is_some());

        app.world_mut().remove_resource::<peek::RoomStreamTarget>();
        app.update();
        assert!(board(&mut app, spectator::SpectateSlot::Stream).is_none());
    }

    #[derive(Component)]
    struct PreviewViewport;

    #[derive(Component)]
    struct StreamViewport;

    const PREVIEW_CELL: starboard::canvas::CellPos = starboard::canvas::CellPos { x: 120, y: 120 };
    const STREAM_CELL: starboard::canvas::CellPos = starboard::canvas::CellPos { x: 700, y: 420 };
    const PREVIEW_RED: starboard::identity::PaletteColor = starboard::identity::PaletteColor(0);
    const STREAM_BLUE: starboard::identity::PaletteColor = starboard::identity::PaletteColor(6);

    fn member(peer: &str, name: &str) -> discovery::RoomMember {
        discovery::RoomMember {
            peer: identity::Peer {
                id: identity::PeerId::from(peer),
                addrs: Vec::new(),
            },
            updated_at: time::UnixTime::EPOCH,
            display_name: Some(identity::Username::from(name)),
            membership: discovery::RoomMembership::Occupant,
        }
    }

    fn label_status() -> DiscoveryState {
        let lobby = discovery::Lobby::from([
            (
                discovery::RoomName::from("alpha"),
                discovery::Room {
                    room_size: discovery::RoomSize::DEFAULT,
                    created_at: time::UnixTime::EPOCH,
                    members: discovery::Members::from([(
                        identity::PeerId::from("peer-a"),
                        member("peer-a", "preview-ada"),
                    )]),
                },
            ),
            (
                discovery::RoomName::from("beta"),
                discovery::Room {
                    room_size: discovery::RoomSize::DEFAULT,
                    created_at: time::UnixTime::EPOCH,
                    members: discovery::Members::from([(
                        identity::PeerId::from("peer-a"),
                        member("peer-a", "stream-ada"),
                    )]),
                },
            ),
        ]);
        freenet_libp2p_bevy_plugin::lobby_room::testing::DiscoveryStateBuilder::new(
            identity::PeerId::from("me"),
        )
        .with_lobby(lobby)
        .build()
    }

    fn painted(
        cell: starboard::canvas::CellPos,
        color: starboard::identity::PaletteColor,
    ) -> starboard::canvas::Canvas {
        let mut canvas = starboard::canvas::Canvas::default();
        for dy in 0..48_u16 {
            for dx in 0..48_u16 {
                let _ = starboard::canvas::write_cell(
                    &mut canvas,
                    starboard::canvas::CellPos {
                        x: cell.x.saturating_add(dx),
                        y: cell.y.saturating_add(dy),
                    },
                    starboard::canvas::Layer {
                        rank: 1,
                        author: 1,
                        color: Some(color),
                        mask: starboard::canvas::FULL_MASK,
                    },
                );
            }
        }
        canvas
    }

    fn spawn_viewports(mut commands: Commands) {
        let root = commands
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .id();
        commands.spawn((
            ChildOf(root),
            PreviewViewport,
            Node {
                width: Val::Px(320.0),
                height: Val::Px(200.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
        ));
        commands.spawn((
            ChildOf(root),
            StreamViewport,
            Node {
                width: Val::Px(1280.0),
                height: Val::Px(800.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
        ));
    }

    fn target_texture(world: &mut World, width: u32, height: u32) -> Handle<Image> {
        world
            .resource_mut::<Assets<Image>>()
            .add(Image::new_target_texture(
                width,
                height,
                bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb,
                None,
            ))
    }

    fn bind_viewport<Marker: Component>(world: &mut World, image: Handle<Image>) {
        let mut viewports = world.query_filtered::<Entity, With<Marker>>();
        if let Some(viewport) = viewports.iter(world).next() {
            world.entity_mut(viewport).insert(ImageNode::new(image));
        }
    }

    fn show_preview_slot(world: &mut World) {
        let image = target_texture(world, 320, 200);
        world.insert_resource(peek::RoomPreviewTarget {
            zone: identity::ZoneName::from("alpha"),
            image: image.clone(),
        });
        world.spawn(spectator::Spectated {
            slot: spectator::SpectateSlot::Preview,
            zone: identity::ZoneName::from("alpha"),
            state: painted(PREVIEW_CELL, PREVIEW_RED),
            freshness: spectator::SpectateFreshness::Live,
        });
        world
            .resource_mut::<starboard::preview::SlotCursors>()
            .entry(spectator::SpectateSlot::Preview)
            .or_default()
            .insert(
                identity::PeerId::from("peer-a"),
                starboard::room::CursorUpdate {
                    cell: starboard::canvas::CellPos { x: 700, y: 500 },
                    size_index: starboard::canvas::DEFAULT_SIZE_INDEX,
                },
            );
        bind_viewport::<PreviewViewport>(world, image);
    }

    fn show_stream_slot(world: &mut World) {
        let image = target_texture(world, 1280, 800);
        world.insert_resource(peek::RoomStreamTarget {
            zone: identity::ZoneName::from("beta"),
            image: image.clone(),
        });
        world.spawn(spectator::Spectated {
            slot: spectator::SpectateSlot::Stream,
            zone: identity::ZoneName::from("beta"),
            state: painted(STREAM_CELL, STREAM_BLUE),
            freshness: spectator::SpectateFreshness::Live,
        });
        world
            .resource_mut::<starboard::preview::SlotCursors>()
            .entry(spectator::SpectateSlot::Stream)
            .or_default()
            .insert(
                identity::PeerId::from("peer-a"),
                starboard::room::CursorUpdate {
                    cell: starboard::canvas::CellPos { x: 700, y: 500 },
                    size_index: starboard::canvas::DEFAULT_SIZE_INDEX,
                },
            );
        bind_viewport::<StreamViewport>(world, image);
    }

    fn show_both_slots(world: &mut World) {
        show_preview_slot(world);
        show_stream_slot(world);
    }

    fn slots_build(app: &mut App) {
        app.insert_resource(label_status());
        app.init_resource::<starboard::preview::SlotCursors>()
            .init_resource::<starboard::room::RoomBrushes>();
        app.add_systems(Startup, spawn_viewports);
        app.add_systems(
            Update,
            (
                starboard::preview::bevy_systems::sync_slot_cameras,
                sync_slot_boards,
                starboard::preview::bevy_systems::paint_slot_board,
                starboard::preview::bevy_systems::draw_slot_remote_cursors,
                starboard::render::bevy_systems::draw_cursor_labels,
            )
                .chain(),
        );
    }

    fn config() -> lele_bevy_preview::preview::Config {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::preview::Config {
            width: 1660,
            height: 860,
            out_dir: out,
            ..Default::default()
        }
    }

    fn scene() -> lele_bevy_preview::scene::Scene {
        lele_bevy_preview::scene::Scene {
            name: String::from("starboard_slot_renders"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: slots_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("preview slot with an unlabelled cursor"),
                    apply: show_preview_slot,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("stream slot with a member-labelled cursor"),
                    apply: show_stream_slot,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("both slots at once, different boards"),
                    apply: show_both_slots,
                },
            ],
            timeline: None,
        }
    }

    #[test]
    fn sync_slot_boards_ui_png_preview() {
        scene().run(&config()).expect("preview");
    }
}
