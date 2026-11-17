use std::collections::BTreeMap;

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use discovery::state_machine::DiscoveryState;
use freenet_libp2p_bevy_plugin::lobby_room::{discovery, spectator};
use freenet_libp2p_bevy_plugin::time;

use crate::starboard;

pub fn draw_slot_remote_cursors(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    status: Option<Res<DiscoveryState>>,
    spectated: Query<&spectator::Spectated<starboard::canvas::Canvas>>,
    cursors: Res<starboard::preview::SlotCursors>,
    brushes: Res<starboard::room::RoomBrushes>,
    mut sprites: starboard::render::CursorSprites,
) {
    let status = status.map(Res::into_inner);
    let cursors = cursors.into_inner();
    let brushes = brushes.into_inner();
    for config in starboard::preview::SLOT_RENDERS {
        let empty = BTreeMap::new();
        let slot_cursors = cursors.get(&config.slot).unwrap_or(&empty);
        let room_brushes = spectated
            .iter()
            .find(|spectated| spectated.slot == config.slot)
            .and_then(|spectated| {
                status.and_then(|status| {
                    status
                        .lobby
                        .get(&discovery::RoomName::from(spectated.zone.clone()))
                })
            })
            .map(|room| {
                starboard::identity::room_brushes(
                    room.members.keys(),
                    room.created_at,
                    time::UnixTime::now(),
                )
            });
        let slot_brushes = room_brushes.as_ref().unwrap_or(brushes);
        let view = starboard::render::CanvasView::fit(Vec2::splat(config.board_side));
        starboard::render::bevy_systems::draw_cursor_sprites(
            &mut commands,
            &mut images,
            &mut sprites,
            slot_cursors,
            slot_brushes,
            &view,
            Vec2::splat(config.board_side),
            config.placement,
            config.labels,
            &RenderLayers::layer(config.layer),
        );
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use discovery::state_machine::DiscoveryState;
    use freenet_libp2p_bevy_plugin::identity;
    use freenet_libp2p_bevy_plugin::lobby_room::{discovery, spectator};
    use freenet_libp2p_bevy_plugin::time;

    use super::draw_slot_remote_cursors;
    use crate::starboard;

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

    fn lobby_status() -> DiscoveryState {
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

    fn cursor() -> starboard::room::CursorUpdate {
        starboard::room::CursorUpdate {
            cell: starboard::canvas::CellPos { x: 700, y: 500 },
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
        app.insert_resource(lobby_status());
        app.init_resource::<starboard::preview::SlotCursors>();
        app.init_resource::<starboard::room::RoomBrushes>();
        app.add_systems(
            Update,
            (
                draw_slot_remote_cursors,
                starboard::render::bevy_systems::draw_cursor_labels,
            )
                .chain(),
        );
        app
    }

    fn spectate(app: &mut App, slot: spectator::SpectateSlot, zone: &str) {
        app.world_mut().spawn(spectator::Spectated {
            slot,
            zone: identity::ZoneName::from(zone),
            state: starboard::canvas::Canvas::default(),
            freshness: spectator::SpectateFreshness::Live,
        });
    }

    fn show_cursor(app: &mut App, slot: spectator::SpectateSlot) {
        let peer = identity::PeerId::from("peer-a");
        app.world_mut()
            .resource_mut::<starboard::preview::SlotCursors>()
            .entry(slot)
            .or_default()
            .insert(peer, cursor());
    }

    fn placement_count(app: &mut App, placement: starboard::render::CursorPlacement) -> usize {
        let world = app.world_mut();
        let mut sprites = world.query::<&starboard::render::CursorPlacement>();
        sprites
            .iter(world)
            .filter(|current| **current == placement)
            .count()
    }

    fn label_texts(app: &mut App) -> Vec<String> {
        let world = app.world_mut();
        let mut labels = world.query::<&Text2d>();
        labels
            .iter(world)
            .map(|text| text.as_str().to_string())
            .filter(|text| !text.is_empty())
            .collect()
    }

    #[test]
    fn test_usage() {
        let mut app = app();
        spectate(&mut app, spectator::SpectateSlot::Preview, "alpha");
        show_cursor(&mut app, spectator::SpectateSlot::Preview);
        app.update();

        assert_eq!(
            placement_count(&mut app, starboard::render::CursorPlacement::Preview),
            1
        );
        assert!(
            label_texts(&mut app).is_empty(),
            "the preview slot never labels a cursor"
        );
    }

    #[test]
    fn the_stream_slot_labels_its_cursor_with_the_member_name() {
        let mut app = app();
        spectate(&mut app, spectator::SpectateSlot::Stream, "beta");
        show_cursor(&mut app, spectator::SpectateSlot::Stream);
        app.update();

        assert_eq!(
            placement_count(&mut app, starboard::render::CursorPlacement::Stream),
            1
        );
        assert_eq!(label_texts(&mut app), vec![String::from("stream-ada")]);
    }

    #[test]
    fn both_slots_draw_their_own_cursor_and_only_the_stream_is_labelled() {
        let mut app = app();
        spectate(&mut app, spectator::SpectateSlot::Preview, "alpha");
        spectate(&mut app, spectator::SpectateSlot::Stream, "beta");
        show_cursor(&mut app, spectator::SpectateSlot::Preview);
        show_cursor(&mut app, spectator::SpectateSlot::Stream);
        app.update();

        assert_eq!(
            placement_count(&mut app, starboard::render::CursorPlacement::Preview),
            1
        );
        assert_eq!(
            placement_count(&mut app, starboard::render::CursorPlacement::Stream),
            1
        );
        assert_eq!(label_texts(&mut app), vec![String::from("stream-ada")]);
    }

    #[test]
    fn dropping_a_slot_despawns_only_its_cursor() {
        let mut app = app();
        spectate(&mut app, spectator::SpectateSlot::Preview, "alpha");
        spectate(&mut app, spectator::SpectateSlot::Stream, "beta");
        show_cursor(&mut app, spectator::SpectateSlot::Preview);
        show_cursor(&mut app, spectator::SpectateSlot::Stream);
        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<spectator::Spectated<starboard::canvas::Canvas>>>();
        let previews: Vec<Entity> = query
            .iter(app.world())
            .filter(|entity| {
                app.world()
                    .get::<spectator::Spectated<starboard::canvas::Canvas>>(*entity)
                    .is_some_and(|spectated| spectated.slot == spectator::SpectateSlot::Preview)
            })
            .collect();
        for entity in previews {
            app.world_mut().despawn(entity);
        }
        app.world_mut()
            .resource_mut::<starboard::preview::SlotCursors>()
            .remove(&spectator::SpectateSlot::Preview);
        app.update();

        assert_eq!(
            placement_count(&mut app, starboard::render::CursorPlacement::Preview),
            0
        );
        assert_eq!(
            placement_count(&mut app, starboard::render::CursorPlacement::Stream),
            1
        );
    }
}
