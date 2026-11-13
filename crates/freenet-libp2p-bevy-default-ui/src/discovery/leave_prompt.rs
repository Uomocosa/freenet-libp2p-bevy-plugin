use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::*;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct LeavePrompt;

#[atomic_delegates]
impl LeavePrompt {
    pub(crate) fn answer_button(marker: impl Component, label: &str, color: Color) -> impl Bundle {}
}

// no test_usage necessary
