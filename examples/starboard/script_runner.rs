use std::time::Duration;

use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::discovery;

use crate::script::StarboardScriptStep;
use crate::shared::script::poll_shared_script_step::{SharedStepView, poll_shared_script_step};
use crate::shared::script::script_runner::{
    ScriptRunner, StepClock, StepOutcome, app_exit_for_verdict,
};
use crate::shared::script::shared_step_effect_sink::SharedStepEffectSink;
use crate::starboard;

/// How long the canvas must keep one hash before `wait-canvas-stable` is done.
const CANVAS_STABLE_FOR: Duration = Duration::from_secs(3);

/// The canvas hash `wait-canvas-stable` is watching, and since when it has held.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanvasHashSince {
    pub hash: u64,
    pub since: Duration,
}

pub type StarboardScriptRunner = ScriptRunner<StarboardScriptStep>;

pub fn run_script(
    time: Res<Time>,
    status: Res<discovery::Status>,
    join_error: Option<Res<discovery::ui::JoinError>>,
    canvas: Res<starboard::canvas::Canvas>,
    brush: Res<starboard::input::OwnBrush>,
    mut runner: ResMut<StarboardScriptRunner>,
    mut draws: MessageWriter<starboard::input::PendingLocalDraw>,
    mut effects: SharedStepEffectSink,
    mut exits: MessageWriter<AppExit>,
    mut stable: Local<Option<CanvasHashSince>>,
) {
    let status = status.into_inner();
    let canvas = canvas.into_inner();
    let brush = *brush.into_inner();
    let view = SharedStepView {
        status,
        join_error: join_error
            .map(Res::into_inner)
            .and_then(|error| error.as_deref()),
        room_message_senders: 0,
    };
    let verdict = runner.advance(time.into_inner().delta(), |step, clock| match step {
        StarboardScriptStep::Shared(shared) => {
            let poll = poll_shared_script_step(shared, clock, &view);
            if let Some(effect) = poll.effect {
                effects.apply(effect);
            }
            poll.outcome
        }
        StarboardScriptStep::PaintFromTo(stroke) => {
            let action = starboard::canvas::DrawAction::Paint(brush.color);
            draw_stroke(action, *stroke, status, brush, &mut draws)
        }
        StarboardScriptStep::EraseFromTo(stroke) => draw_stroke(
            starboard::canvas::DrawAction::Erase,
            *stroke,
            status,
            brush,
            &mut draws,
        ),
        StarboardScriptStep::WaitCanvasNonEmpty => {
            if canvas.tiles.iter().any(Option::is_some) {
                StepOutcome::Done
            } else {
                StepOutcome::Pending
            }
        }
        StarboardScriptStep::WaitCanvasStable => {
            is_canvas_stable(&mut stable, starboard::canvas::hash_canvas(canvas), clock)
        }
        StarboardScriptStep::WaitCellPainted(pos) => {
            if starboard::canvas::read_cell(canvas, *pos).color.is_some() {
                StepOutcome::Done
            } else {
                StepOutcome::Pending
            }
        }
    });
    if let Some(exit) = app_exit_for_verdict(verdict) {
        exits.write(exit);
    }
}

// needed helper: draws one scripted stroke; drawing outside a room is a script error
fn draw_stroke(
    action: starboard::canvas::DrawAction,
    stroke: starboard::canvas::Stroke,
    status: &discovery::Status,
    brush: starboard::input::OwnBrush,
    draws: &mut MessageWriter<starboard::input::PendingLocalDraw>,
) -> StepOutcome {
    if !starboard::room::is_in_room(status) {
        return StepOutcome::Failed("cannot draw outside a room".to_string());
    }
    draws.write(starboard::input::PendingLocalDraw(
        starboard::input::draw_op(brush, action, stroke),
    ));
    StepOutcome::Done
}

// needed helper: whether the canvas hash has held for CANVAS_STABLE_FOR within this step
fn is_canvas_stable(
    stable: &mut Option<CanvasHashSince>,
    hash: u64,
    clock: StepClock,
) -> StepOutcome {
    let held = match *stable {
        Some(seen) if seen.hash == hash && !clock.is_first_poll => seen,
        _ => CanvasHashSince {
            hash,
            since: clock.elapsed,
        },
    };
    *stable = Some(held);
    if clock.elapsed.saturating_sub(held.since) >= CANVAS_STABLE_FOR {
        StepOutcome::Done
    } else {
        StepOutcome::Pending
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::discovery;

    use super::{StarboardScriptRunner, run_script};
    use crate::script::StarboardScriptStep;
    use crate::shared::intent::PendingAction;
    use crate::shared::script::script_runner::ScriptVerdict;
    use crate::shared::script::timed_script_step::parse_timed_script_step;
    use crate::starboard;

    fn app(script: &[&str]) -> App {
        let steps = script
            .iter()
            .map(|raw| parse_timed_script_step::<StarboardScriptStep>(raw).expect("step"))
            .collect();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<discovery::Status>();
        app.init_resource::<starboard::canvas::Canvas>();
        app.init_resource::<starboard::input::OwnBrush>();
        app.insert_resource(PendingAction::default());
        app.add_message::<starboard::input::PendingLocalDraw>();
        app.add_message::<discovery::Command>();
        app.insert_resource(StarboardScriptRunner::new(steps));
        app.add_systems(Update, run_script);
        app
    }

    #[test]
    fn test_usage() {
        let mut app = app(&["sleep:0s"]);
        app.update();
        assert_eq!(app.world().resource::<StarboardScriptRunner>().index, 1);
        assert_eq!(app.world().resource::<Messages<AppExit>>().len(), 1);
    }

    #[test]
    fn painting_outside_a_room_fails_the_script() {
        let mut app = app(&["paint-from-to:10.10-20.10"]);
        app.update();
        assert_eq!(
            app.world().resource::<StarboardScriptRunner>().verdict,
            ScriptVerdict::Failed
        );
    }

    #[test]
    fn a_join_step_arms_the_intent_with_the_room() {
        let mut app = app(&["join:room-1", "stay"]);
        app.update();
        let pending = app.world().resource::<PendingAction>();
        assert!(matches!(
            pending.0,
            Some(crate::shared::action::Action::Join(_))
        ));
    }

    #[test]
    fn a_cell_wait_is_done_once_the_cell_is_painted() {
        let mut app = app(&["wait-cell-painted:10.20@30s"]);
        app.update();
        assert_eq!(app.world().resource::<StarboardScriptRunner>().index, 0);
        let mut canvas = app.world_mut().resource_mut::<starboard::canvas::Canvas>();
        let _ = starboard::canvas::write_cell(
            &mut canvas,
            starboard::canvas::CellPos { x: 10, y: 20 },
            starboard::canvas::Cell {
                lamport: 1,
                author: 1,
                color: Some(starboard::input::OwnBrush::default().color),
            },
        );
        app.update();
        assert_eq!(app.world().resource::<StarboardScriptRunner>().index, 1);
    }

    #[test]
    fn a_still_canvas_is_stable_after_the_hold() {
        let mut app = app(&["wait-canvas-stable@30s"]);
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_millis(200),
        ));
        for _ in 0..20 {
            app.update();
        }
        assert_eq!(app.world().resource::<StarboardScriptRunner>().index, 1);
    }
}
