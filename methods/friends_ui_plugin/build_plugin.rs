use bevy::prelude::*;

use crate::direct_dial;

pub fn build_plugin(_plugin: &direct_dial::ui::FriendsUiPlugin, app: &mut App) {
    use direct_dial::ui::bevy_systems as friends;
    app.add_systems(Startup, friends::spawn_friends_tab);
    app.add_systems(Update, friends::rebuild_friend_list);
    app.add_systems(Update, friends::gate_add_friend);
    app.add_systems(
        Update,
        (
            friends::handle_add_friend,
            friends::show_friend_address_error,
        )
            .chain(),
    );
    app.add_systems(Update, friends::handle_remove_friend);
    app.add_systems(Update, friends::refresh_own_address);
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::build_plugin;
    use crate::direct_dial;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(direct_dial::Config::from(None));
        app.insert_resource(direct_dial::PeerBook::default());
        app.insert_resource(direct_dial::FriendAddressError(None));
        build_plugin(&direct_dial::ui::FriendsUiPlugin, &mut app);
        app.update();
        let mut tabs = app
            .world_mut()
            .query_filtered::<(), With<direct_dial::ui::FriendsTab>>();
        assert_eq!(tabs.iter(app.world()).count(), 1);
    }

    #[test]
    fn ui_adds_a_row_through_real_button_and_input() {
        use bevy::text::EditableText;
        use bevy::ui::Pressed;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(direct_dial::Config::from(None));
        app.insert_resource(direct_dial::PeerBook::default());
        app.insert_resource(direct_dial::FriendAddressError(None));
        build_plugin(&direct_dial::ui::FriendsUiPlugin, &mut app);
        app.update();
        let id = libp2p::PeerId::random();
        let text = format!("/ip4/127.0.0.1/tcp/4001/p2p/{id}");
        {
            let world = app.world_mut();
            let mut inputs = world
                .query_filtered::<&mut EditableText, With<direct_dial::ui::FriendAddressInput>>();
            if let Ok(mut input) = inputs.single_mut(world) {
                *input = EditableText::new(text);
            } else {
                panic!("friend input missing");
            }
        }
        app.update();
        let button = app
            .world_mut()
            .query_filtered::<Entity, With<direct_dial::ui::AddFriendButton>>()
            .single(app.world())
            .expect("add button");
        app.world_mut().entity_mut(button).insert(Pressed);
        app.update();
        app.update();
        let mut rows = app
            .world_mut()
            .query_filtered::<(), With<direct_dial::ui::FriendRow>>();
        assert_eq!(rows.iter(app.world()).count(), 1);
    }
}
