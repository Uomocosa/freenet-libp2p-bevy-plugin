use crate::identity;
use crate::testing;
use crate::time;

#[derive(Debug, Clone, Copy, Default)]
pub struct NoFakeNetworkKit;

#[rustfmt::skip]
impl testing::traits::FakeNetworkKit for NoFakeNetworkKit {
    fn on_hello(&mut self, _peer: &identity::Peer, _game: &identity::GameId, _now: time::UnixTime) -> Vec<Vec<u8>> { Vec::new() }
    fn on_extension(&mut self, _from: &identity::PeerId, _game: &identity::GameId, _bytes: Vec<u8>, _now: time::UnixTime) -> Vec<testing::FanOut> { Vec::new() }
    fn on_disconnect(&mut self, _peer: &identity::PeerId, _game: &identity::GameId, _now: time::UnixTime) -> Vec<testing::FanOut> { Vec::new() }
}

#[cfg(test)]
mod tests {
    use crate::testing;

    #[test]
    fn test_usage() {
        let _ = testing::NoFakeNetworkKit;
    }
}
