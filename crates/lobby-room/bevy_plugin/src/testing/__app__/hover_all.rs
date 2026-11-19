use bevy::picking::hover::Hovered;
use bevy::prelude::{Entity, With, World};

use crate::testing;
use crate::ui;

pub fn hover_all(world: &mut World) {
    let mut inputs =
        world.query_filtered::<&mut bevy::text::EditableText, With<ui::discovery::RoomNameInput>>();
    if let Ok(mut input) = inputs.single_mut(world) {
        input.queue_edit(bevy::text::TextEdit::Insert("gamma".into()));
    }
    let mut glows = world.query_filtered::<Entity, With<ui::discovery::HoverGlow>>();
    let hovered: Vec<Entity> = glows.iter(world).collect();
    for entity in hovered {
        world.entity_mut(entity).insert((
            Hovered(true),
            testing::PinHovered,
            ui::discovery::HoverGlow(1.0),
        ));
    }
}

#[cfg(test)]
mod tests {
    use bevy::picking::hover::Hovered;
    use bevy::prelude::World;

    use crate::ui;

    use super::hover_all;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        let glow = world
            .spawn((ui::discovery::HoverGlow::default(), Hovered(false)))
            .id();
        hover_all(&mut world);
        assert_eq!(
            **world.get::<ui::discovery::HoverGlow>(glow).expect("glow"),
            1.0
        );
        assert!(world.get::<Hovered>(glow).is_some_and(Hovered::get));
    }
}
