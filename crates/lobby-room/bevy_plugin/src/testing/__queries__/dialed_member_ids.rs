use crate::p2p;
use crate::testing;

#[must_use]
pub fn dialed_member_ids(me: &str) -> Vec<String> {
    let mut app = testing::peek_app(me, &[("alpha", 10)]);
    testing::rendezvous_room(&mut app);
    testing::select_room(&mut app, Some("alpha"));
    app.update();
    testing::drained_messages::<p2p::Libp2pCommand>(&mut app)
        .into_iter()
        .filter_map(|command| match command {
            p2p::Libp2pCommand::Dial(peer) => Some(peer.id.as_str().to_string()),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::dialed_member_ids;

    #[test]
    fn test_usage() {
        assert_eq!(dialed_member_ids("spectator").len(), 1);
    }
}
