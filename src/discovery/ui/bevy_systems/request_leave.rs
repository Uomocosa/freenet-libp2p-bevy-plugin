use bevy::prelude::*;

use crate::discovery;
use discovery::ui::{LeaveDoor, LeavePromptOpen};

pub fn request_leave(
    snapshot: Res<discovery::Snapshot>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    doors: Query<&Interaction, (Changed<Interaction>, With<LeaveDoor>)>,
    mut prompt: ResMut<LeavePromptOpen>,
) {
    if snapshot.into_inner().room.is_none() {
        return;
    }
    let clicked = doors
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    let escaped = keys.is_some_and(|keys| keys.just_pressed(KeyCode::Escape));
    let open = match (clicked, escaped) {
        (true, _) => true,
        (false, true) => !**prompt,
        (false, false) => return,
    };
    if **prompt != open {
        tracing::info!(target: "discovery", "ui leave prompt open={open}");
        **prompt = open;
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::request_leave;
    use crate::{discovery, identity};

    fn app_in_room() -> App {
        let mut app = App::new();
        app.insert_resource(discovery::Snapshot {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                members: discovery::Members::new(),
            }),
            ..Default::default()
        });
        app.init_resource::<discovery::ui::LeavePromptOpen>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, request_leave);
        app
    }

    fn is_open(app: &App) -> bool {
        **app.world().resource::<discovery::ui::LeavePromptOpen>()
    }

    #[test]
    fn test_usage() {
        let mut clicked = app_in_room();
        clicked
            .world_mut()
            .spawn((discovery::ui::LeaveDoor, Interaction::Pressed));
        clicked.update();
        assert!(is_open(&clicked));

        let mut escaped = app_in_room();
        escaped
            .world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        escaped.update();
        assert!(is_open(&escaped));
    }
}
