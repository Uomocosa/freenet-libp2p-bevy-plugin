use bevy::prelude::Reflect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum LinkStatus {
    Connected,
    Lost,
}
