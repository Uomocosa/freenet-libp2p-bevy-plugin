use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::{JoinError, JoinErrorDismissButton};

pub fn dismiss_join_failed(
    buttons: Query<(), (Added<Pressed>, With<JoinErrorDismissButton>)>,
    mut join_error: ResMut<JoinError>,
) {
    if buttons.iter().next().is_some() {
        **join_error = None;
    }
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::dismiss_join_failed;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.insert_resource(ui::discovery::JoinError(Some(String::from("err"))));
        app.world_mut()
            .spawn((ui::discovery::JoinErrorDismissButton, Pressed));
        app.add_systems(Update, dismiss_join_failed);
        app.update();
        assert_eq!(**app.world().resource::<ui::discovery::JoinError>(), None);
    }
}
