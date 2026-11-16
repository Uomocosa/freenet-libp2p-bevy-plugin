use atomic_delegate_macros::atomic_delegates;

pub struct DayMillis;

#[atomic_delegates]
impl DayMillis {
    pub fn now() -> u64 {}
}

#[cfg(test)]
mod tests {
    use super::DayMillis;
    use crate::testing;

    #[test]
    fn test_usage() {
        assert!(DayMillis::now() < testing::harness::MILLIS_PER_DAY);
    }
}
