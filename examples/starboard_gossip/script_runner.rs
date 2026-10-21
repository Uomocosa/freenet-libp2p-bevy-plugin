use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::{discovery, identity};

use crate::script::ScriptStep;
use crate::shared::action::Action;
use crate::starboard;

/// How long the game keeps running after it published its quit, so the tombstone lands.
const QUIT_PUBLISH_GRACE_SECONDS: f32 = 2.0;

/// The scripted session and how far it has got.
#[derive(Resource, Debug, Clone)]
pub struct ScriptRunner {
    pub steps: Vec<ScriptStep>,
    pub index: usize,
    pub waited_secs: f32,
    pub last_canvas_hash: Option<u64>,
    pub is_started: bool,
}

impl ScriptRunner {
    #[must_use]
    pub const fn new(steps: Vec<ScriptStep>) -> Self {
        Self {
            steps,
            index: 0,
            waited_secs: 0.0,
            last_canvas_hash: None,
            is_started: false,
        }
    }
}

pub fn run_script(
    time: Res<Time>,
    status: Res<discovery::Status>,
    canvas: Res<starboard::canvas::Canvas>,
    brush: Res<starboard::input::OwnBrush>,
    mut runner: ResMut<ScriptRunner>,
    mut draws: MessageWriter<starboard::input::PendingLocalDraw>,
    mut commands: MessageWriter<discovery::Command>,
    mut pending_action: ResMut<crate::shared::intent::PendingAction>,
    mut exits: MessageWriter<AppExit>,
) {
    let time = time.into_inner();
    let status = status.into_inner();
    let canvas = canvas.into_inner();
    let brush = brush.into_inner();
    runner.waited_secs += time.delta_secs();
    loop {
        let Some(step) = runner.steps.get(runner.index).cloned() else {
            return;
        };
        let label = step.label();
        let mut should_exit = false;
        match step {
            ScriptStep::CreateRoom(room) => {
                if !runner.is_started {
                    pending_action.0 = Some(Action::Create(room.clone()));
                    runner.is_started = true;
                }
                if !is_in_room(status, &room) {
                    return;
                }
            }
            ScriptStep::JoinRoom(room) => {
                if !runner.is_started {
                    pending_action.0 = Some(Action::Join(room.clone()));
                    runner.is_started = true;
                }
                if !is_in_room(status, &room) {
                    return;
                }
            }
            ScriptStep::LeaveRoom(room) => {
                if !runner.is_started {
                    commands.write(discovery::Command::Leave);
                    runner.is_started = true;
                }
                if is_in_room(status, &room) {
                    return;
                }
            }
            ScriptStep::WaitForNMembers(count) => {
                let connected = u32::try_from(connected_members(status)).unwrap_or(u32::MAX);
                if connected < *count {
                    return;
                }
            }
            ScriptStep::WaitSecs(duration) => {
                if runner.waited_secs < duration.as_secs_f32() {
                    return;
                }
            }
            ScriptStep::WaitConverged(room) => {
                if !is_in_room(status, &room) {
                    return;
                }
                let hash = starboard::canvas::hash_canvas(canvas);
                let is_stable = runner.last_canvas_hash == Some(hash);
                runner.last_canvas_hash = Some(hash);
                if !is_stable {
                    return;
                }
            }
            ScriptStep::PaintFromTo(stroke) => {
                if !runner.is_started {
                    let action = starboard::canvas::DrawAction::Paint(brush.color);
                    if !draw_stroke(action, stroke, status, canvas, *brush, &mut draws) {
                        return;
                    }
                    runner.is_started = true;
                }
            }
            ScriptStep::EraseFromTo(stroke) => {
                if !runner.is_started {
                    let action = starboard::canvas::DrawAction::Erase;
                    if !draw_stroke(action, stroke, status, canvas, *brush, &mut draws) {
                        return;
                    }
                    runner.is_started = true;
                }
            }
            ScriptStep::ExitGame => {
                if !runner.is_started {
                    commands.write(discovery::Command::Quit);
                    runner.is_started = true;
                }
                if runner.waited_secs < QUIT_PUBLISH_GRACE_SECONDS {
                    return;
                }
                should_exit = true;
            }
        }
        tracing::info!("starboard script done {label}");
        if should_exit {
            exits.write(AppExit::Success);
            return;
        }
        runner.index = runner.index.saturating_add(1);
        runner.is_started = false;
        runner.waited_secs = 0.0;
        runner.last_canvas_hash = None;
    }
}

// needed helper: draws one scripted stroke, when the peer may draw it
fn draw_stroke(
    action: starboard::canvas::DrawAction,
    stroke: starboard::canvas::Stroke,
    status: &discovery::Status,
    canvas: &starboard::canvas::Canvas,
    brush: starboard::input::OwnBrush,
    draws: &mut MessageWriter<starboard::input::PendingLocalDraw>,
) -> bool {
    if !starboard::room::is_in_room(status) {
        return false;
    }
    if matches!(action, starboard::canvas::DrawAction::Erase)
        && !canvas.tiles.iter().any(Option::is_some)
    {
        return false;
    }
    draws.write(starboard::input::PendingLocalDraw(
        starboard::input::draw_op(brush, action, stroke),
    ));
    true
}

// needed helper: whether the peer is in the named room
fn is_in_room(status: &discovery::Status, room: &identity::RoomName) -> bool {
    status
        .room
        .as_ref()
        .is_some_and(|joined| joined.name == *room)
}

// needed helper: how many peers are connected in the room the peer is in
fn connected_members(status: &discovery::Status) -> usize {
    status.room.as_ref().map_or(0, |room| {
        room.members
            .values()
            .filter(|member| member.status == discovery::LinkStatus::Connected)
            .count()
    })
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::discovery;

    use super::{ScriptRunner, run_script};
    use crate::script::ScriptStep;
    use crate::shared::intent::PendingAction;
    use crate::starboard;

    fn app(steps: Vec<ScriptStep>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<discovery::Status>();
        app.init_resource::<starboard::canvas::Canvas>();
        app.init_resource::<starboard::input::OwnBrush>();
        app.insert_resource(PendingAction::default());
        app.add_message::<starboard::input::PendingLocalDraw>();
        app.add_message::<discovery::Command>();
        app.insert_resource(ScriptRunner::new(steps));
        app.add_systems(Update, run_script);
        app
    }

    #[test]
    fn test_usage() {
        let mut app = app(vec![ScriptStep::WaitSecs(std::time::Duration::ZERO)]);
        app.update();
        assert_eq!(
            app.world().resource::<ScriptRunner>().index,
            1,
            "a zero-second wait is done on the first frame"
        );
    }

    #[test]
    fn a_paint_step_waits_for_the_room() {
        let mut app = app(vec![ScriptStep::PaintFromTo(starboard::canvas::Stroke {
            from: starboard::canvas::CellPos { x: 10, y: 10 },
            to: starboard::canvas::CellPos { x: 20, y: 10 },
        })]);
        app.update();
        assert_eq!(app.world().resource::<ScriptRunner>().index, 0);
        assert_eq!(
            app.world()
                .resource::<Messages<starboard::input::PendingLocalDraw>>()
                .len(),
            0
        );
    }

    #[test]
    fn a_join_step_arms_the_intent_with_the_room() {
        let mut app = app(vec![ScriptStep::JoinRoom(
            freenet_libp2p_bevy_plugin::identity::RoomName::from("room-1"),
        )]);
        app.update();
        let pending = app.world().resource::<PendingAction>();
        assert!(matches!(
            pending.0,
            Some(crate::shared::action::Action::Join(_))
        ));
    }
}
