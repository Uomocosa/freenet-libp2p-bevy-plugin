use atomic_delegate_macros::atomic_delegates;

pub struct UnixNanos;

#[atomic_delegates]
impl UnixNanos {
    pub fn now() -> u128 {}
}

#[cfg(test)]
mod tests {
    use super::UnixNanos;

    #[test]
    fn test_usage() {
        assert!(UnixNanos::now() > 0);
    }
}
