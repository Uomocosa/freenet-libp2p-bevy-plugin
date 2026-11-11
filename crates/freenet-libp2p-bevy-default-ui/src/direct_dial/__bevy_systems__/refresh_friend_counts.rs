use bevy::prelude::*;

use crate::direct_dial;
use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;

pub fn refresh_friend_counts(
    view: Option<Res<core_direct_dial::FriendsView>>,
    mut texts: Query<&mut Text, With<direct_dial::FriendsCountText>>,
) {
    let view: Vec<core_direct_dial::FriendView> =
        view.map(|view| (**view).clone()).unwrap_or_default();
    let online = view
        .iter()
        .filter(|friend| friend.presence != core_direct_dial::FriendPresence::Offline)
        .count();
    let label = format!("FRIENDS  {online} online / {}", view.len());
    for mut text in &mut texts {
        *text = Text::new(label.clone());
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::refresh_friend_counts;
    use crate::direct_dial;
    use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
    use freenet_libp2p_bevy_core::identity;

    fn friend_view(
        id: &str,
        presence: core_direct_dial::FriendPresence,
    ) -> core_direct_dial::FriendView {
        core_direct_dial::FriendView {
            id: identity::PeerId::from(id),
            name: id.to_string(),
            presence,
        }
    }

    fn app_with(view: core_direct_dial::FriendsView) -> App {
        let mut app = App::new();
        app.insert_resource(view);
        app.world_mut()
            .spawn((direct_dial::FriendsCountText, Text::new("old")));
        app.add_systems(Update, refresh_friend_counts);
        app
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(core_direct_dial::FriendsView(vec![
            friend_view("alice", core_direct_dial::FriendPresence::Offline),
            friend_view("bob", core_direct_dial::FriendPresence::InLobby),
        ]));
        app.update();
        let mut texts = app
            .world_mut()
            .query_filtered::<&Text, With<direct_dial::FriendsCountText>>();
        let shown: Vec<String> = texts
            .iter(app.world())
            .map(|text| text.as_str().to_string())
            .collect();
        assert_eq!(shown, vec![String::from("FRIENDS  1 online / 2")]);
    }

    #[test]
    fn the_header_counts_online_and_total() {
        let mut app = app_with(core_direct_dial::FriendsView(vec![
            friend_view("alice", core_direct_dial::FriendPresence::Offline),
            friend_view("bob", core_direct_dial::FriendPresence::InLobby),
            friend_view(
                "carol",
                core_direct_dial::FriendPresence::InRoom(identity::RoomName::from("room-7")),
            ),
        ]));
        app.update();
        let mut texts = app
            .world_mut()
            .query_filtered::<&Text, With<direct_dial::FriendsCountText>>();
        let shown: Vec<String> = texts
            .iter(app.world())
            .map(|text| text.as_str().to_string())
            .collect();
        assert_eq!(shown, vec![String::from("FRIENDS  2 online / 3")]);
    }

    #[test]
    fn without_the_view_the_header_shows_zero() {
        let mut app = App::new();
        app.world_mut()
            .spawn((direct_dial::FriendsCountText, Text::new("old")));
        app.add_systems(Update, refresh_friend_counts);
        app.update();
        let mut texts = app
            .world_mut()
            .query_filtered::<&Text, With<direct_dial::FriendsCountText>>();
        let shown: Vec<String> = texts
            .iter(app.world())
            .map(|text| text.as_str().to_string())
            .collect();
        assert_eq!(shown, vec![String::from("FRIENDS  0 online / 0")]);
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(preview_view());
        app.add_systems(Startup, spawn_count);
        app.add_systems(Update, refresh_friend_counts);
    }

    fn preview_view() -> core_direct_dial::FriendsView {
        core_direct_dial::FriendsView(vec![
            friend_view("alice", core_direct_dial::FriendPresence::Offline),
            friend_view("bob", core_direct_dial::FriendPresence::InLobby),
        ])
    }

    fn spawn_count(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands.spawn((
            ChildOf(root),
            direct_dial::FriendsCountText,
            Text::new("old"),
            TextColor(direct_dial::DRAWER_DIM_COLOR),
        ));
    }

    const fn preview_noop(_world: &mut World) {}

    fn clear_view(world: &mut World) {
        *world.resource_mut::<core_direct_dial::FriendsView>() =
            core_direct_dial::FriendsView(Vec::new());
    }

    #[test]
    fn refresh_friend_counts_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_friend_counts"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("one online of two"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("everyone gone"),
                    apply: clear_view,
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
