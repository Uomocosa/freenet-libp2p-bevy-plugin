use std::time::Duration;

use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::{discovery, identity};

/// The exit code the game reports when its join failed.
pub const JOIN_FAILED_EXIT_CODE: u8 = 2;

/// The room the peer is trying to be in.
#[derive(Resource, Debug, Clone)]
pub struct JoinTarget(pub identity::RoomName);

/// The limits of the join watchdog: how long the peer waits for its room to
/// show up in the lobby, and how long it keeps running after it reported the
/// failure, so the error screen is on screen and the peer is back in the lobby.
#[derive(Resource, Debug, Clone, Copy)]
pub struct JoinLimits {
    pub timeout: Duration,
    pub grace: Duration,
}

pub fn exit_on_join_failure(
    status: Res<discovery::Status>,
    join_error: Res<discovery::ui::JoinError>,
    target: Res<JoinTarget>,
    limits: Res<JoinLimits>,
    time: Res<Time>,
    mut exits: MessageWriter<AppExit>,
    mut elapsed: Local<f32>,
    mut grace: Local<f32>,
    mut is_reported: Local<bool>,
) {
    if *is_reported {
        return;
    }
    let time = time.into_inner();
    let status = status.into_inner();
    let join_error = join_error.into_inner();
    let target = target.into_inner();
    let limits = limits.into_inner();
    *elapsed += time.delta_secs();
    let Some(reason) = failure_reason(status, join_error, target) else {
        return;
    };
    if join_error.is_none() && *elapsed < limits.timeout.as_secs_f32() {
        return;
    }
    *grace += time.delta_secs();
    if *grace < limits.grace.as_secs_f32() {
        return;
    }
    tracing::info!(
        "starboard join failed room={} reason={reason}",
        target.0.as_str()
    );
    *is_reported = true;
    exits.write(AppExit::from_code(JOIN_FAILED_EXIT_CODE));
}

// needed helper: why the join failed, if it did
fn failure_reason(
    status: &discovery::Status,
    join_error: &discovery::ui::JoinError,
    target: &JoinTarget,
) -> Option<String> {
    if let Some(message) = join_error.as_deref() {
        return Some(message.to_string());
    }
    if status.room.is_some() || !status.is_lobby_attached {
        return None;
    }
    if status.lobby.contains_key(&target.0) {
        return None;
    }
    Some(format!(
        "Room {} no longer exists (all players left)",
        target.0.as_str()
    ))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{discovery, identity};

    use super::{JoinLimits, JoinTarget, exit_on_join_failure};

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<discovery::Status>();
        app.init_resource::<discovery::ui::JoinError>();
        app.insert_resource(JoinTarget(identity::RoomName::from("room-1")));
        app.insert_resource(JoinLimits {
            timeout: Duration::ZERO,
            grace: Duration::ZERO,
        });
        app.add_systems(Update, exit_on_join_failure);
        app
    }

    #[test]
    fn test_usage() {
        let mut app = app();
        app.update();
        assert!(
            app.world().resource::<Messages<AppExit>>().is_empty(),
            "a peer that never asked for a room does not report a failed join"
        );
    }

    #[test]
    fn a_room_that_left_the_lobby_is_reported() {
        let mut app = app();
        app.insert_resource(discovery::Status {
            is_lobby_attached: true,
            ..discovery::Status::default()
        });
        app.update();
        app.update();
        assert_eq!(app.world().resource::<Messages<AppExit>>().len(), 1);
    }

    #[test]
    fn the_plugin_join_error_is_reported() {
        let mut app = app();
        **app.world_mut().resource_mut::<discovery::ui::JoinError>() = Some(String::from(
            "Room room-1 no longer exists (all players left)",
        ));
        app.update();
        app.update();
        assert_eq!(app.world().resource::<Messages<AppExit>>().len(), 1);
    }
}
