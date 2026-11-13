use bevy::prelude::*;
use bevy::ui_widgets::Button;

pub fn answer_button(marker: impl Component, label: &str, color: Color) -> impl Bundle {
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

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::answer_button;

    #[derive(Component)]
    struct Marker;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        let entity = world
            .spawn(answer_button(Marker, "Leave", Color::WHITE))
            .id();
        assert!(world.get::<Marker>(entity).is_some());
        assert!(world.get::<Node>(entity).is_some());
    }
}
