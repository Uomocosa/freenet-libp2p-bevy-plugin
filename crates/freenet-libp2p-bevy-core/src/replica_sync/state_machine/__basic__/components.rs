use bevy::prelude::Component;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ZoneLiveness {
    pub is_zone_still_listed: bool,
    pub has_other_history_holders: bool,
}
