use bevy::prelude::Reflect;
use derive_more::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum LinkStatus {
    Connected,
    Lost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum Step {
    #[display("fetch")]
    Fetch,
    #[display("deploy")]
    Deploy,
}
