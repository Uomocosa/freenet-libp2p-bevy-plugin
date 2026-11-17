use crate::spectator;

pub fn reset<Snap>(bootstrap: &mut spectator::HistoryBootstrap<Snap>) {
    bootstrap.prefix = None;
    bootstrap.manifest = None;
    bootstrap.chunks.clear();
}

#[cfg(test)]
mod tests {
    use super::reset;
    use crate::p2p;
    use crate::spectator;

    #[test]
    fn test_usage() {
        let mut bootstrap = spectator::HistoryBootstrap::<Vec<u8>> {
            prefix: Some(p2p::HistoryKeyPrefix::from("prefix")),
            ..Default::default()
        };
        bootstrap.chunks.insert(p2p::ChunkIndex(0), vec![1]);
        reset(&mut bootstrap);
        assert!(bootstrap.prefix.is_none());
        assert!(bootstrap.chunks.is_empty());
    }
}
