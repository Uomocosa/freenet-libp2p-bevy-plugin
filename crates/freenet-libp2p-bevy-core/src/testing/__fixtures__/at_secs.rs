use crate::replica_sync;

#[must_use]
pub fn at_secs(seconds: u64) -> replica_sync::ElapsedSinceAppStart {
    replica_sync::ElapsedSinceAppStart::from(std::time::Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests {
    use crate::replica_sync;

    use super::at_secs;

    #[test]
    fn test_usage() {
        assert_eq!(
            at_secs(7),
            replica_sync::ElapsedSinceAppStart::from(std::time::Duration::from_secs(7))
        );
    }
}
