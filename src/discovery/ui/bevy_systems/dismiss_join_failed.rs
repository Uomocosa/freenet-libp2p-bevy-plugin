use bevy::prelude::*;

use crate::discovery;
use discovery::ui::{JoinError, JoinErrorDismissButton};

pub fn dismiss_join_failed(
    buttons: Query<&Interaction, (Changed<Interaction>, With<JoinErrorDismissButton>)>,
    mut join_error: ResMut<JoinError>,
) {
    if buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        **join_error = None;
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::dismiss_join_failed;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.insert_resource(discovery::ui::JoinError(Some(String::from("err"))));
        app.world_mut()
            .spawn((discovery::ui::JoinErrorDismissButton, Interaction::Pressed));
        app.add_systems(Update, dismiss_join_failed);
        app.update();
        assert_eq!(**app.world().resource::<discovery::ui::JoinError>(), None);
    }
}
