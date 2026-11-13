use crate::direct_dial;
use crate::p2p;

impl From<&direct_dial::state_machine::Output> for Option<p2p::Libp2pCommand> {
    fn from(output: &direct_dial::state_machine::Output) -> Self {
        match output {
            direct_dial::state_machine::Output::Dial(peer) => {
                Some(p2p::Libp2pCommand::Dial(peer.clone()))
            }
            direct_dial::state_machine::Output::SavePeerBook(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::direct_dial;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let dial = direct_dial::state_machine::Output::Dial(identity::Peer {
            id: identity::PeerId::from("alice"),
            addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
        });
        assert!(matches!(
            Option::<p2p::Libp2pCommand>::from(&dial),
            Some(p2p::Libp2pCommand::Dial(peer)) if peer.id == identity::PeerId::from("alice")
        ));
        let save =
            direct_dial::state_machine::Output::SavePeerBook(direct_dial::PeerBook::default());
        assert_eq!(Option::<p2p::Libp2pCommand>::from(&save), None);
    }
}
