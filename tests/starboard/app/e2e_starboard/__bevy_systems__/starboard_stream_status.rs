use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use freenet_libp2p_bevy_plugin::lobby_room::{peek, spectator, ui};
use serde_json::Value;

use crate::e2e_starboard::StreamStatus;

pub fn starboard_stream_status(
    In(_params): In<Option<Value>>,
    spectated: Query<&spectator::Spectated<starboard::starboard::canvas::Canvas>>,
    stream_cursors: Option<Res<starboard::starboard::preview::SlotCursors>>,
    target: Option<Res<peek::RoomStreamTarget>>,
    images: Query<&ImageNode, With<ui::discovery::RoomStreamImage>>,
) -> BrpResult {
    let has_stream_image =
        target.is_some_and(|target| images.iter().any(|node| node.image == target.image));
    let spectated = spectated
        .iter()
        .find(|spectated| spectated.slot == spectator::SpectateSlot::Stream);
    let status = StreamStatus {
        streamed_room: spectated.map(|spectated| spectated.zone.to_string()),
        freshness: spectated.map(|spectated| format!("{:?}", spectated.freshness)),
        painted_tiles: spectated.map_or(0, |spectated| {
            spectated
                .state
                .tiles
                .iter()
                .filter(|tile| tile.is_some())
                .count()
        }),
        remote_cursors: stream_cursors
            .and_then(|cursors| {
                cursors
                    .get(&spectator::SpectateSlot::Stream)
                    .map(std::collections::BTreeMap::len)
            })
            .unwrap_or(0),
        has_stream_image,
    };
    serde_json::to_value(status).map_err(BrpError::internal)
}

#[cfg(test)]
mod tests {
    use bevy::asset::RenderAssetUsages;
    use bevy::prelude::*;
    use bevy::remote::{RemoteMethods, RemotePlugin};
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    use freenet_libp2p_bevy_plugin::identity;
    use freenet_libp2p_bevy_plugin::lobby_room::{peek, spectator, ui};

    use crate::e2e_starboard::StreamStatus;

    fn status(app: &mut App) -> Option<StreamStatus> {
        let system = app
            .world_mut()
            .register_system(super::starboard_stream_status);
        let result = app.world_mut().run_system_with(system, None);
        result
            .ok()
            .and_then(Result::ok)
            .and_then(|value| serde_json::from_value(value).ok())
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(
            RemotePlugin::default()
                .with_method_main("starboard/stream_status", super::starboard_stream_status),
        );
        let methods = app.world().resource::<RemoteMethods>();
        assert!(methods.get("starboard/stream_status").is_some());
        assert_eq!(
            status(&mut app),
            Some(StreamStatus {
                streamed_room: None,
                freshness: None,
                painted_tiles: 0,
                remote_cursors: 0,
                has_stream_image: false,
            })
        );
    }

    #[test]
    fn a_live_streamed_room_with_its_image_reports_showing() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Image>>();
        let zone = identity::ZoneName::from("alpha");
        let image = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(Image::new_fill(
                Extent3d {
                    width: 4,
                    height: 4,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                &[0, 0, 0, 255],
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            ));
        let mut canvas = starboard::starboard::canvas::Canvas::default();
        let _ = starboard::starboard::canvas::write_cell(
            &mut canvas,
            starboard::starboard::canvas::CellPos { x: 3, y: 3 },
            starboard::starboard::canvas::Layer {
                rank: 1,
                author: 1,
                color: Some(starboard::starboard::identity::PaletteColor(0)),
                mask: starboard::starboard::canvas::FULL_MASK,
            },
        );
        app.world_mut().spawn(spectator::Spectated {
            slot: spectator::SpectateSlot::Stream,
            zone,
            state: canvas,
            freshness: spectator::SpectateFreshness::Live,
        });
        app.insert_resource(peek::RoomStreamTarget {
            zone: identity::ZoneName::from("alpha"),
            image: image.clone(),
        });
        app.world_mut()
            .spawn((ui::discovery::RoomStreamImage, ImageNode::new(image)));

        let status = status(&mut app).expect("stream status");
        assert!(status.is_showing_room("alpha"));
        assert_eq!(status.painted_tiles, 1);
        assert!(matches!(status.streamed_room.as_deref(), Some("alpha")));
    }
}
