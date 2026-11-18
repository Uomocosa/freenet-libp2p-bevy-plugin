use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::lobby_room::discovery;

use crate::starboard;

#[must_use]
pub fn spawn_friend_highlight(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    friend: &discovery::RoomFriend,
    at: Vec3,
    layers: &RenderLayers,
) -> Entity {
    let side = starboard::preview::FRIEND_HIGHLIGHT_RING_PX;
    let image = images.add(starboard::render::square_image(
        u32::from(side),
        starboard::preview::friend_ring(side, starboard::preview::FRIEND_HIGHLIGHT_BAND_PX).pixels,
    ));
    let highlight = commands
        .spawn((
            starboard::preview::FriendHighlight(friend.peer.clone()),
            Sprite {
                image,
                custom_size: Some(Vec2::splat(f32::from(side))),
                ..default()
            },
            Transform::from_xyz(at.x, at.y, starboard::preview::FRIEND_HIGHLIGHT_Z),
            layers.clone(),
        ))
        .id();
    let label = commands
        .spawn((
            starboard::render::cursor_label(
                starboard::preview::FRIEND_LABEL_COLOR,
                friend.name.as_str(),
            ),
            ChildOf(highlight),
            layers.clone(),
        ))
        .id();
    commands.entity(label).insert(Transform::from_xyz(
        0.0,
        f32::from(side).mul_add(0.5, starboard::render::LABEL_GAP_PX),
        starboard::render::LABEL_Z,
    ));
    highlight
}

#[cfg(test)]
mod tests {
    use bevy::camera::visibility::RenderLayers;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::identity;
    use freenet_libp2p_bevy_plugin::lobby_room::discovery;

    use super::spawn_friend_highlight;
    use crate::starboard;

    fn friend() -> discovery::RoomFriend {
        discovery::RoomFriend {
            peer: identity::PeerId::from("peer-a"),
            name: identity::Username::from("alice"),
        }
    }

    fn spawn_test(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
        let _ = spawn_friend_highlight(
            &mut commands,
            &mut images,
            &friend(),
            Vec3::new(10.0, 20.0, starboard::preview::FRIEND_HIGHLIGHT_Z),
            &RenderLayers::default(),
        );
    }

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default_nearest(),
        ));
        app.add_systems(Startup, spawn_test);
        app
    }

    #[test]
    fn test_usage() {
        let mut app = app();
        app.update();

        let mut highlights = app.world_mut().query::<(
            Entity,
            &starboard::preview::FriendHighlight,
            &Sprite,
            &Transform,
        )>();
        let (highlight, marker, sprite, transform) = highlights
            .single(app.world())
            .expect("one friend highlight");
        assert_eq!(**marker, friend().peer);
        assert_eq!(
            sprite.custom_size,
            Some(Vec2::splat(f32::from(
                starboard::preview::FRIEND_HIGHLIGHT_RING_PX
            )))
        );
        assert!((transform.translation.x - 10.0).abs() < 0.000_1);
        assert!((transform.translation.z - starboard::preview::FRIEND_HIGHLIGHT_Z).abs() < 0.000_1);

        let mut labels = app.world_mut().query::<(&Text2d, &TextColor, &ChildOf)>();
        let (text, color, _child_of) = labels
            .iter(app.world())
            .find(|(_, _, child_of)| child_of.parent() == highlight)
            .expect("the friend name label");
        assert_eq!(text.as_str(), "alice");
        assert_eq!(*color, TextColor(starboard::preview::FRIEND_LABEL_COLOR));
    }
}
