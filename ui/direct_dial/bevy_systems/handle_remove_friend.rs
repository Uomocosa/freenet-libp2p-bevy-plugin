use bevy::prelude::*;
use bevy::ui::Pressed;

use crate::direct_dial;
use crate::ui;
use ui::direct_dial::FriendRemoveButton;

pub fn handle_remove_friend(
    buttons: Query<&FriendRemoveButton, Added<Pressed>>,
    mut book: ResMut<direct_dial::PeerBook>,
    config: Res<direct_dial::Config>,
) {
    let config = config.into_inner();
    for button in &buttons {
        if book.remove_friend(button)
            && let Some(path) = config.as_ref()
            && let Err(error) = book.save_to_file(path)
        {
            tracing::error!(target: "direct_dial", "friend save failed: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::handle_remove_friend;
    use crate::direct_dial;
    use crate::identity;
    use crate::ui;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let mut book = direct_dial::PeerBook::default();
        assert!(book.add_friend(identity::Peer {
            id: identity::PeerId::from("alice"),
            addrs: Vec::new(),
        }));
        app.insert_resource(book);
        app.insert_resource(direct_dial::Config::from(None));
        app.world_mut().spawn((
            ui::direct_dial::FriendRemoveButton(identity::PeerId::from("alice")),
            Pressed,
        ));
        app.add_systems(Update, handle_remove_friend);
        app.update();
        assert!(app.world().resource::<direct_dial::PeerBook>().is_empty());
    }
}
