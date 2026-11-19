use std::collections::BTreeMap;

use bevy::picking::hover::Hovered;
use bevy::prelude::{ChildOf, Entity, World};

use crate::discovery;
use crate::identity;
use crate::testing;
use crate::ui;

pub fn hover_alpha_pill(world: &mut World) {
    testing::show_room_preview(world, "alpha");
    if let Some(row) = testing::room_row(world, "alpha") {
        let mut pills = world.query::<(Entity, &ChildOf, &ui::discovery::RoomBubble)>();
        let alpha_pills: Vec<Entity> = pills
            .iter(world)
            .filter(|(_entity, link, _bubble)| link.parent() == row)
            .map(|(entity, _link, _bubble)| entity)
            .collect();
        for pill in alpha_pills {
            world
                .entity_mut(pill)
                .insert((Hovered(true), testing::PinHovered));
        }
    }
    **world.resource_mut::<ui::discovery::HoveredOccupancyPill>() =
        Some(discovery::RoomName::from("alpha"));
    world.insert_resource(discovery::RoomFriendNames(BTreeMap::from([(
        discovery::RoomName::from("alpha"),
        vec![
            identity::Username::from("alice"),
            identity::Username::from("bob"),
        ],
    )])));
}

#[cfg(test)]
mod tests {
    use bevy::picking::hover::Hovered;
    use bevy::prelude::{App, Assets, ChildOf, Image};

    use crate::discovery;
    use crate::ui;

    use super::hover_alpha_pill;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>();
        app.init_resource::<ui::discovery::SelectedRoom>();
        app.init_resource::<ui::discovery::HoveredOccupancyPill>();
        let row = app
            .world_mut()
            .spawn(ui::discovery::RoomButton(discovery::RoomName::from(
                "alpha",
            )))
            .id();
        let pill = app
            .world_mut()
            .spawn((ChildOf(row), ui::discovery::RoomBubble))
            .id();
        hover_alpha_pill(app.world_mut());
        assert_eq!(
            **app
                .world()
                .resource::<ui::discovery::HoveredOccupancyPill>(),
            Some(discovery::RoomName::from("alpha"))
        );
        assert!(app.world().get::<Hovered>(pill).is_some_and(Hovered::get));
    }
}
