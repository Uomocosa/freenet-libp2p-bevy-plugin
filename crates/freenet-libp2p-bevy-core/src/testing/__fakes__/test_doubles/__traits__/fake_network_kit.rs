use crate::identity;
use crate::testing;
use crate::time;

pub trait FakeNetworkKit: Send + 'static {
    fn on_hello(
        &mut self,
        peer: &identity::Peer,
        game: &identity::GameId,
        now: time::UnixTime,
    ) -> Vec<Vec<u8>>;

    fn on_extension(
        &mut self,
        from: &identity::PeerId,
        game: &identity::GameId,
        bytes: Vec<u8>,
        now: time::UnixTime,
    ) -> Vec<testing::FanOut>;

    fn on_disconnect(
        &mut self,
        peer: &identity::PeerId,
        game: &identity::GameId,
        now: time::UnixTime,
    ) -> Vec<testing::FanOut>;
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::testing;
    use crate::testing::traits::FakeNetworkKit;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut kit = testing::NoFakeNetworkKit;
        let peer = testing::peer("a", &[]);
        let game = identity::GameId::from("game");
        assert_eq!(
            kit.on_hello(&peer, &game, time::UnixTime::from_secs(1)),
            Vec::<Vec<u8>>::new()
        );
        assert_eq!(
            kit.on_disconnect(&peer.id, &game, time::UnixTime::from_secs(1)),
            Vec::new()
        );
    }
}
