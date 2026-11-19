use bevy::ecs::component::Component;
use bevy::prelude::{App, Entity, With};
use bevy::ui::Pressed;

pub fn press_marker<C: Component>(app: &mut App) {
    let mut buttons = app.world_mut().query_filtered::<Entity, With<C>>();
    if let Some(button) = buttons.iter(app.world()).next() {
        app.world_mut().entity_mut(button).insert(Pressed);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::App;
    use bevy::ui::Pressed;

    use crate::ui;

    use super::press_marker;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        let button = app
            .world_mut()
            .spawn(ui::discovery::EscMenuFriendsButton)
            .id();
        press_marker::<ui::discovery::EscMenuFriendsButton>(&mut app);
        assert!(app.world().get::<Pressed>(button).is_some());
    }
}
