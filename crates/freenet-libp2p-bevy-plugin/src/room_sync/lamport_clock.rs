use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default, Deref, DerefMut)]
pub struct LamportClock(pub u32);

#[rustfmt::skip]
impl LamportClock {
    pub fn tick(&mut self) -> u32 { **self = (*self).saturating_add(1); **self }
    pub fn observe(&mut self, remote: u32) { **self = (*self).max(remote).saturating_add(1); }
}

#[cfg(test)]
mod tests {
    use super::LamportClock;

    #[test]
    fn test_usage() {
        let mut clock = LamportClock::default();
        assert_eq!(clock.tick(), 1);
        assert_eq!(clock.tick(), 2);
        clock.observe(10);
        assert_eq!(clock.tick(), 12);
        clock.observe(3);
        assert_eq!(clock.tick(), 14);
    }

    #[test]
    fn observe_takes_the_maximum_and_ticks() {
        let mut clock = LamportClock::default();
        clock.observe(5);
        assert_eq!(*clock, 6);
        clock.observe(9);
        assert_eq!(*clock, 10);
        clock.observe(3);
        assert_eq!(*clock, 11);
    }
}
