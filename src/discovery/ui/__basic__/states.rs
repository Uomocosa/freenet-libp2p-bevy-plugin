use bevy::prelude::{States, SubStates};

#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Screen {
    #[default]
    Connecting,
    Lobby,
    InRoom,
}

#[derive(SubStates, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[source(Screen = Screen::InRoom)]
pub enum LeaveDialog {
    #[default]
    Closed,
    Open,
}
