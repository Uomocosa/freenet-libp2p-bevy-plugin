use crate::replica_sync;

#[must_use]
pub const fn meta(lamport: u32, author: u16) -> replica_sync::OpMeta {
    replica_sync::OpMeta { lamport, author }
}

#[cfg(test)]
mod tests {
    use super::meta;

    #[test]
    fn test_usage() {
        let op_meta = meta(3, 2);
        assert_eq!(op_meta.lamport, 3);
        assert_eq!(op_meta.author, 2);
    }
}
