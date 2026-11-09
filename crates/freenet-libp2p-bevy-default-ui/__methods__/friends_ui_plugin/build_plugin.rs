use bevy::prelude::*;

use crate::direct_dial;

pub fn build_plugin(_plugin: &direct_dial::FriendsUiPlugin, app: &mut App) {
    use direct_dial::bevy_systems as friends;
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
    use lele_bevy_state_machine::StateMachine;

    use super::build_plugin;
    use crate::direct_dial;
    use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
    use freenet_libp2p_bevy_core::time;

    fn empty_state() -> core_direct_dial::state_machine::DialState {
        core_direct_dial::state_machine::DialState::from_peer_book(
            &core_direct_dial::PeerBook::default(),
            time::UnixTime::from_secs(10),
        )
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<core_direct_dial::state_machine::Input>();
        app.insert_resource(empty_state());
        app.insert_resource(core_direct_dial::FriendAddressError(None));
        build_plugin(&direct_dial::FriendsUiPlugin, &mut app);
        app.update();
        let mut tabs = app
            .world_mut()
            .query_filtered::<(), With<direct_dial::FriendsTab>>();
        assert_eq!(tabs.iter(app.world()).count(), 1);
    }

    #[test]
    fn ui_adds_a_row_through_real_button_and_input() {
        use bevy::text::EditableText;
        use bevy::ui::Pressed;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<core_direct_dial::state_machine::Input>();
        app.insert_resource(empty_state());
        app.insert_resource(core_direct_dial::FriendAddressError(None));
        build_plugin(&direct_dial::FriendsUiPlugin, &mut app);
        app.update();
        let id = libp2p::PeerId::random();
        let text = format!("/ip4/127.0.0.1/tcp/4001/p2p/{id}");
        {
            let world = app.world_mut();
            let mut inputs =
                world.query_filtered::<&mut EditableText, With<direct_dial::FriendAddressInput>>();
            if let Ok(mut input) = inputs.single_mut(world) {
                *input = EditableText::new(text);
            } else {
                panic!("friend input missing");
            }
        }
        app.update();
        let button = app
            .world_mut()
            .query_filtered::<Entity, With<direct_dial::AddFriendButton>>()
            .single(app.world())
            .expect("add button");
        app.world_mut().entity_mut(button).insert(Pressed);
        app.update();
        let pending: Vec<core_direct_dial::state_machine::Input> = app
            .world_mut()
            .resource_mut::<Messages<core_direct_dial::state_machine::Input>>()
            .drain()
            .collect();
        assert_eq!(pending.len(), 1);
        let now = time::UnixTime::from_secs(10);
        for input in pending {
            app.world_mut()
                .resource_mut::<core_direct_dial::state_machine::DialState>()
                .update(input, now);
        }
        app.update();
        app.update();
        let mut rows = app
            .world_mut()
            .query_filtered::<(), With<direct_dial::FriendRow>>();
        assert_eq!(rows.iter(app.world()).count(), 1);
    }
}
