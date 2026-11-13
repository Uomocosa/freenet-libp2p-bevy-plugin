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
    use crate::time;

    struct SilentKit;

    impl testing::traits::FakeNetworkKit for SilentKit {
        fn on_hello(
            &mut self,
            _peer: &identity::Peer,
            _game: &identity::GameId,
            _now: time::UnixTime,
        ) -> Vec<Vec<u8>> {
            Vec::new()
        }

        fn on_extension(
            &mut self,
            _from: &identity::PeerId,
            _game: &identity::GameId,
            _bytes: Vec<u8>,
            _now: time::UnixTime,
        ) -> Vec<testing::FanOut> {
            Vec::new()
        }

        fn on_disconnect(
            &mut self,
            _peer: &identity::PeerId,
            _game: &identity::GameId,
            _now: time::UnixTime,
        ) -> Vec<testing::FanOut> {
            Vec::new()
        }
    }

    #[test]
    fn test_usage() {
        use crate::testing::traits::FakeNetworkKit;
        let mut kit = SilentKit;
        let peer = identity::Peer {
            id: identity::PeerId::from("a"),
            addrs: Vec::new(),
        };
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
