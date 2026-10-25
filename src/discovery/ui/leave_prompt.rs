use bevy::prelude::*;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct LeavePrompt;

#[rustfmt::skip]
impl LeavePrompt {
    pub(crate) fn answer_button(marker: impl Component, label: &str, color: Color) -> impl Bundle {
        (
            marker,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(18.0), Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(color),
            children![Text::new(label)],
        )
    }
}

// no test_usage necessary
