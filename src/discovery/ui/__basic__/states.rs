use bevy::prelude::{States, SubStates};

#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiScreen {
    #[default]
    Connecting,
    Lobby,
    InRoom,
}

#[derive(SubStates, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[source(UiScreen = UiScreen::InRoom)]
pub enum LeaveDialog {
    #[default]
    Closed,
    Open,
}
