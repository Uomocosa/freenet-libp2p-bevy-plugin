use bevy::prelude::*;

use crate::direct_dial;
use crate::identity;
use crate::p2p;

pub fn update_own_share_address(
    ready: Option<Res<p2p::ReadyChannel>>,
    mut share: ResMut<direct_dial::OwnShareAddress>,
) {
    let address = ready.and_then(|ready| {
        let me = ready.ready_rx.borrow().as_ref().cloned()?;
        let observed: Vec<identity::PeerAddr> = ready
            .observed_rx
            .borrow()
            .as_ref()
            .map(|addrs| addrs.as_slice().to_vec())
            .unwrap_or_default();
        Some(direct_dial::OwnAddress::from_own_peer(&me, &observed))
    });
    **share = address;
}
