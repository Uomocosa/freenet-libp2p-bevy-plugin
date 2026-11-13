use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

pub fn button(marker: impl Component, label: &str, hint: &str) -> impl Bundle {
    (
        marker,
        Button,
        Hovered::default(),
        ui::discovery::HoverGlow::default(),
        UiTransform::default(),
        Outline::new(
            Val::Px(ui::discovery::HOVER_OUTLINE_WIDTH),
            Val::ZERO,
            Color::NONE,
        ),
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
            border_radius: BorderRadius::all(Val::Px(ui::discovery::MENU_BUTTON_RADIUS)),
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        },
        BackgroundColor(ui::discovery::BUTTON_COLOR),
        children![
            (
                Text::new(label),
                TextColor(ui::discovery::BUTTON_LABEL_COLOR),
                TextFont {
                    font_size: FontSize::Px(ui::discovery::MENU_LABEL_FONT_SIZE),
                    ..default()
                },
            ),
            (Node {
                flex_grow: 1.0,
                ..default()
            },),
            (
                Text::new(hint),
                TextColor(ui::constants::DIM_TEXT_COLOR),
                TextFont {
                    font_size: FontSize::Px(ui::discovery::MENU_HINT_FONT_SIZE),
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
