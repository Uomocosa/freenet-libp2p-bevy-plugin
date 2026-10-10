#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::discovery;

#[derive(Resource)]
pub struct Username(pub String);

#[derive(Component)]
pub struct StatusText;

pub fn setup_ui(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        StatusText,
        Text::new("lobby starting"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

/// Event-driven `lobby mesh` marker: logs the live link count the moment it changes.
pub fn log_mesh(snapshot: Res<discovery::Snapshot>, mut last: Local<Option<(String, usize)>>) {
    if !snapshot.is_changed() {
        return;
    }
    let Some(room) = snapshot.room.as_ref() else {
        return;
    };
    let connected = room
        .members
        .values()
        .filter(|member| member.status == discovery::LinkStatus::Connected)
        .count();
    let now = (room.name.as_str().to_string(), connected);
    if last.as_ref() == Some(&now) {
        return;
    }
    tracing::info!(
        "lobby mesh room={} connected={connected}",
        room.name.as_str()
    );
    *last = Some(now);
}

pub fn log_tick(
    time: Res<Time>,
    username: Res<Username>,
    snapshot: Res<discovery::Snapshot>,
    mut accumulator: Local<f32>,
    mut query: Query<&mut Text, With<StatusText>>,
) {
    *accumulator += time.delta_secs();
    if *accumulator < 1.0 {
        return;
    }
    *accumulator = 0.0;
    let rooms: Vec<&str> = snapshot.lobby.keys().map(|name| name.as_str()).collect();
    let catalogue = if rooms.is_empty() {
        "none".to_string()
    } else {
        rooms.join(",")
    };
    let room = snapshot
        .room
        .as_ref()
        .map_or_else(|| "none".to_string(), |room| room.name.as_str().to_string());
    let members = snapshot.room.as_ref().map_or(0, |room| room.members.len());
    let connected = snapshot.room.as_ref().map_or(0, |room| {
        room.members
            .values()
            .filter(|member| member.status == discovery::LinkStatus::Connected)
            .count()
    });
    tracing::info!(
        "lobby tick catalogue={catalogue} room={room} members={members} connected={connected}"
    );
    if let Ok(mut text) = query.single_mut() {
        *text = Text::new(format!(
            "{} room={room} members={members} connected={connected} rooms=[{catalogue}]",
            username.0
        ));
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::net_id;

    use super::{StatusText, Username, log_tick};
    use freenet_libp2p_bevy_plugin::discovery;

    fn preview_build(app: &mut App) {
        app.insert_resource(Username(String::from("preview")));
        app.insert_resource(discovery::Snapshot::default());
        app.add_systems(Startup, spawn_status);
        app.add_systems(Update, log_tick);
    }

    fn spawn_status(mut commands: Commands) {
        commands.spawn((
            StatusText,
            Text::new("lobby starting"),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                ..default()
            },
        ));
    }

    const fn preview_noop(_world: &mut World) {}

    fn add_room(world: &mut World) {
        let mut snapshot = world.resource_mut::<discovery::Snapshot>();
        snapshot.lobby.insert(
            net_id::RoomName::from("gamma"),
            discovery::RoomEntry {
                capacity: 4,
                members: std::collections::BTreeMap::new(),
            },
        );
    }

    fn config() -> lele_bevy_preview::preview::Config {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        }
    }

    fn scene(
        timeline: Option<lele_bevy_preview::scene::Timeline>,
    ) -> lele_bevy_preview::scene::Scene {
        lele_bevy_preview::scene::Scene {
            name: String::from("status"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("lobby"),
                apply: preview_noop,
            }],
            timeline,
        }
    }

    #[test]
    #[ignore = "headed preview"]
    fn status_ui_png_preview() {
        let _marker: Option<StatusText> = None;
        lele_bevy_preview::run(&scene(None), &config(), env!("CARGO_PKG_NAME")).expect("preview");
    }

    #[test]
    #[ignore = "headed recording"]
    fn status_ui_mp4_preview() {
        let timeline = lele_bevy_preview::scene::Timeline {
            label: String::from("add_gamma"),
            frames: 90,
            fps: 30,
            apply: add_room,
        };
        lele_bevy_preview::run(&scene(Some(timeline)), &config(), env!("CARGO_PKG_NAME"))
            .expect("preview");
    }
}
