use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use crate::basic::constants::DIM_TEXT_COLOR;
use crate::discovery::{
    BUTTON_COLOR, BUTTON_LABEL_COLOR, HOVER_OUTLINE_WIDTH, HoverGlow, MENU_BUTTON_RADIUS,
    MENU_HINT_FONT_SIZE, MENU_LABEL_FONT_SIZE,
};

pub fn button(marker: impl Component, label: &str, hint: &str) -> impl Bundle {
    (
        marker,
        Button,
        Hovered::default(),
        HoverGlow::default(),
        UiTransform::default(),
        Outline::new(Val::Px(HOVER_OUTLINE_WIDTH), Val::ZERO, Color::NONE),
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
            border_radius: BorderRadius::all(Val::Px(MENU_BUTTON_RADIUS)),
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        },
        BackgroundColor(BUTTON_COLOR),
        children![
            (
                Text::new(label),
                TextColor(BUTTON_LABEL_COLOR),
                TextFont {
                    font_size: FontSize::Px(MENU_LABEL_FONT_SIZE),
                    ..default()
                },
            ),
            (Node {
                flex_grow: 1.0,
                ..default()
            },),
            (
                Text::new(hint),
                TextColor(DIM_TEXT_COLOR),
                TextFont {
                    font_size: FontSize::Px(MENU_HINT_FONT_SIZE),
                    ..default()
                },
            ),
        ],
    )
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::button;

    #[derive(Component)]
    struct Marker;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        let entity = world.spawn(button(Marker, "Friends", "Tab")).id();
        assert!(world.get::<Marker>(entity).is_some());
        assert!(world.get::<Node>(entity).is_some());
    }
}
