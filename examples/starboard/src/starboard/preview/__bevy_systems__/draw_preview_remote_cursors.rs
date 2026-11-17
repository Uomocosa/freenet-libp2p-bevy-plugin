use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use discovery::state_machine::DiscoveryState;
use freenet_libp2p_bevy_plugin::lobby_room::{discovery, spectator};
use freenet_libp2p_bevy_plugin::time;

use crate::starboard;

pub fn draw_preview_remote_cursors(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    status: Option<Res<DiscoveryState>>,
    spectated: Query<&spectator::Spectated<starboard::canvas::Canvas>>,
    cursors: Res<starboard::preview::PreviewCursors>,
    brushes: Res<starboard::room::RoomBrushes>,
    mut sprites: starboard::render::CursorSprites,
) {
    let status = status.map(Res::into_inner);
    let cursors = cursors.into_inner();
    let brushes = brushes.into_inner();
    let view =
        starboard::render::CanvasView::fit(Vec2::splat(starboard::preview::PREVIEW_BOARD_SIDE));
    let window = Vec2::splat(starboard::preview::PREVIEW_BOARD_SIDE);
    let layers = RenderLayers::layer(starboard::preview::PREVIEW_LAYER);
    let Some(spectated) = spectated
        .iter()
        .find(|spectated| spectated.slot == spectator::SpectateSlot::Preview)
    else {
        starboard::render::bevy_systems::draw_cursor_sprites(
            &mut commands,
            &mut images,
            &mut sprites,
            &std::collections::BTreeMap::new(),
            &std::collections::BTreeMap::new(),
            &view,
            window,
            starboard::render::CursorPlacement::Preview,
            &layers,
        );
        return;
    };
    let zone = discovery::RoomName::from(spectated.zone.clone());
    let preview_brushes = status
        .and_then(|status| status.lobby.get(&zone))
        .map(|room| {
            starboard::identity::room_brushes(
                room.members.keys(),
                room.created_at,
                time::UnixTime::now(),
            )
        });
    let brushes = preview_brushes.as_ref().unwrap_or(brushes);
    starboard::render::bevy_systems::draw_cursor_sprites(
        &mut commands,
        &mut images,
        &mut sprites,
        cursors,
        brushes,
        &view,
        window,
        starboard::render::CursorPlacement::Preview,
        &layers,
    );
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use discovery::state_machine::DiscoveryState;
    use freenet_libp2p_bevy_plugin::identity;
    use freenet_libp2p_bevy_plugin::lobby_room::{discovery, spectator};
    use freenet_libp2p_bevy_plugin::time;

    use super::draw_preview_remote_cursors;
    use crate::starboard;

    fn lobby_status() -> DiscoveryState {
        let mut members = discovery::Members::new();
        members.insert(
            identity::PeerId::from("peer-a"),
            discovery::RoomMember {
                peer: identity::Peer {
                    id: identity::PeerId::from("peer-a"),
                    addrs: Vec::new(),
                },
                updated_at: time::UnixTime::EPOCH,
                display_name: None,
                membership: discovery::RoomMembership::Occupant,
            },
        );
        let lobby = discovery::Lobby::from([(
            discovery::RoomName::from("alpha"),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::EPOCH,
                members,
            },
        )]);
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

    fn preview_sprites(app: &mut App) -> usize {
        let world = app.world_mut();
        let mut sprites = world
            .query_filtered::<&starboard::render::CursorPlacement, With<starboard::render::Cursor>>(
            );
        sprites
            .iter(world)
            .filter(|placement| **placement == starboard::render::CursorPlacement::Preview)
            .count()
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default_nearest(),
        ));
        app.insert_resource(lobby_status());
        app.world_mut().spawn(spectator::Spectated {
            slot: spectator::SpectateSlot::Preview,
            zone: identity::ZoneName::from("alpha"),
            state: starboard::canvas::Canvas::default(),
            freshness: spectator::SpectateFreshness::Live,
        });
        app.insert_resource(starboard::preview::PreviewCursors(
            std::collections::BTreeMap::from([(identity::PeerId::from("peer-a"), cursor())]),
        ));
        app.init_resource::<starboard::room::RoomBrushes>();
        app.add_systems(Update, draw_preview_remote_cursors);

        app.update();
        assert_eq!(preview_sprites(&mut app), 1);

        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<spectator::Spectated<starboard::canvas::Canvas>>>();
        let entities: Vec<Entity> = query.iter(app.world()).collect();
        for entity in entities {
            app.world_mut().despawn(entity);
        }
        app.update();
        assert_eq!(preview_sprites(&mut app), 0);
    }
}
