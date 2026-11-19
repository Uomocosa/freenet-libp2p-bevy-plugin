use lele_bevy_state_machine::Now;

use crate::direct_dial;
use crate::p2p;
use crate::testing;
use crate::time;

#[must_use]
pub fn friends_app_with_book(
    book: &direct_dial::PeerBook,
) -> (
    testing::TestApp,
    tokio::sync::broadcast::Sender<p2p::Libp2pEvent>,
) {
    let frozen = time::UnixTime::from_secs(2_000_000_000);
    let mut app = testing::TestApp::new();
    app.insert_resource(p2p::Libp2pEventChannel::default());
    app.insert_resource(Now(frozen));
    let mut app = app.with_plugin(direct_dial::FriendsPlugin::new(direct_dial::Config::from(
        None,
    )));
    app.insert_resource(direct_dial::state_machine::DialState::from_peer_book(
        book, frozen,
    ));
    let tx = app.world().resource::<p2p::Libp2pEventChannel>().tx.clone();
    (app, tx)
}

#[cfg(test)]
mod tests {
    use crate::direct_dial;

    use super::friends_app_with_book;

    #[test]
    fn test_usage() {
        let (app, _tx) = friends_app_with_book(&direct_dial::PeerBook::default());
        assert!(
            app.world()
                .contains_resource::<direct_dial::state_machine::DialState>()
        );
    }
}
