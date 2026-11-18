use crate::ui;
use bevy::prelude::*;

use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;

pub fn refresh_friend_counts(
    view: Option<Res<core_direct_dial::FriendsView>>,
    mut texts: Query<&mut Text, With<ui::direct_dial::FriendsCountText>>,
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
    use crate::ui;
    use bevy::prelude::*;

    use super::refresh_friend_counts;
    use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
    use freenet_libp2p_bevy_core::identity;

    #[test]
    fn refresh_friend_counts_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_friend_counts"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.insert_resource(core_direct_dial::FriendsView(vec![
                    core_direct_dial::FriendView {
                        id: identity::PeerId::from("alice"),
                        name: String::from("alice"),
                        presence: core_direct_dial::FriendPresence::Offline,
                    },
                    core_direct_dial::FriendView {
                        id: identity::PeerId::from("bob"),
                        name: String::from("bob"),
                        presence: core_direct_dial::FriendPresence::Online,
                    },
                ]));
                app.add_systems(Startup, |mut commands: Commands| {
                    let root = commands
                        .spawn(lele_bevy_preview::scene::Scene::centered_root())
                        .id();
                    commands.spawn((
                        ChildOf(root),
                        ui::direct_dial::FriendsCountText,
                        Text::new("old"),
                        TextColor(ui::constants::DIM_TEXT_COLOR),
                    ));
                });
                app.add_systems(Update, refresh_friend_counts);
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("one online of two"),
                    apply: |_world: &mut World| {},
                },
                lele_bevy_preview::scene::State {
                    label: String::from("everyone gone"),
                    apply: |world: &mut World| {
                        *world.resource_mut::<core_direct_dial::FriendsView>() =
                            core_direct_dial::FriendsView(Vec::new());
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
