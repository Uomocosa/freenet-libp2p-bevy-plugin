use std::time::Duration;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::{discovery, testing};

use crate::script::StarboardStep;
use crate::starboard;

/// How long the canvas must keep one hash before `wait-canvas-stable` is done.
const CANVAS_STABLE_FOR: Duration = Duration::from_secs(3);

/// The canvas hash `wait-canvas-stable` is watching, and since when it has held.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanvasHashSince {
    pub hash: u64,
    pub since: Duration,
}

pub type StarboardScriptRunner = testing::ScriptRunner<StarboardStep>;

#[derive(SystemParam)]
pub struct StarboardStepParam<'w, 's> {
    status: Res<'w, discovery::Status>,
    join_error: Option<Res<'w, discovery::ui::JoinError>>,
    canvas: Res<'w, starboard::canvas::Canvas>,
    brush: Res<'w, starboard::input::OwnBrush>,
    draws: MessageWriter<'w, starboard::input::PendingLocalDraw>,
    effects: testing::E2EStepEffectSink<'w>,
    stable: Local<'s, Option<CanvasHashSince>>,
}

impl testing::PollableScriptStep for StarboardStep {
    type Param<'w, 's> = StarboardStepParam<'w, 's>;

    fn poll(
        step: &Self,
        clock: testing::StepClock,
        param: &mut Self::Param<'_, '_>,
    ) -> testing::StepOutcome {
        match step {
            Self::Shared(shared) => {
                let view = testing::E2EStepView {
                    status: &param.status,
                    join_error: param
                        .join_error
                        .as_deref()
                        .and_then(|error| error.as_deref()),
                    room_message_senders: 0,
                };
                let poll = shared.poll_shared_script_step(clock, &view);
                if let Some(effect) = poll.effect {
                    param.effects.apply(effect);
                }
                poll.outcome
            }
            Self::Paint(strokes) => {
                for stroke in strokes {
                    let action = starboard::canvas::DrawAction::Paint(param.brush.color);
                    let outcome = draw_stroke(
                        action,
                        crate::script::to_canvas_stroke(*stroke),
                        &param.status,
                        *param.brush,
                        &mut param.draws,
                    );
                    if outcome != testing::StepOutcome::Done {
                        return outcome;
                    }
                }
                testing::StepOutcome::Done
            }
            Self::Erase(strokes) => {
                for stroke in strokes {
                    let outcome = draw_stroke(
                        starboard::canvas::DrawAction::Erase,
                        crate::script::to_canvas_stroke(*stroke),
                        &param.status,
                        *param.brush,
                        &mut param.draws,
                    );
                    if outcome != testing::StepOutcome::Done {
                        return outcome;
                    }
                }
                testing::StepOutcome::Done
            }
            Self::WaitCanvasNonEmpty => {
                if param.canvas.tiles.iter().any(Option::is_some) {
                    testing::StepOutcome::Done
                } else {
                    testing::StepOutcome::Pending
                }
            }
            Self::WaitCanvasStable => is_canvas_stable(
                &mut param.stable,
                starboard::canvas::hash_canvas(&param.canvas),
                clock,
            ),
            Self::WaitCellPainted(pos) => {
                if starboard::canvas::read_cell(&param.canvas, crate::script::to_canvas_cell(*pos))
                    .color
                    .is_some()
                {
                    testing::StepOutcome::Done
                } else {
                    testing::StepOutcome::Pending
                }
            }
        }
    }
}

// needed helper: draws one scripted stroke; drawing outside a room is a script error
fn draw_stroke(
    action: starboard::canvas::DrawAction,
    stroke: starboard::canvas::Stroke,
    status: &discovery::Status,
    brush: starboard::input::OwnBrush,
    draws: &mut MessageWriter<starboard::input::PendingLocalDraw>,
) -> testing::StepOutcome {
    if !starboard::room::is_in_room(status) {
        return testing::StepOutcome::Failed("cannot draw outside a room".to_string());
    }
    draws.write(starboard::input::PendingLocalDraw(
        starboard::input::draw_op(brush, action, stroke),
    ));
    testing::StepOutcome::Done
}

// needed helper: whether the canvas hash has held for CANVAS_STABLE_FOR within this step
fn is_canvas_stable(
    stable: &mut Option<CanvasHashSince>,
    hash: u64,
    clock: testing::StepClock,
) -> testing::StepOutcome {
    let held = match *stable {
        Some(seen) if seen.hash == hash && !clock.is_first_poll => seen,
        _ => CanvasHashSince {
            hash,
            since: clock.elapsed,
        },
    };
    *stable = Some(held);
    if clock.elapsed.saturating_sub(held.since) >= CANVAS_STABLE_FOR {
        testing::StepOutcome::Done
    } else {
        testing::StepOutcome::Pending
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{discovery, testing};

    use super::StarboardScriptRunner;
    use crate::script::StarboardStep;
    use crate::starboard;

    fn app(script: &[&str]) -> App {
        let steps = script
            .iter()
            .map(|raw| testing::TimedScriptStep::<StarboardStep>::try_from(*raw).expect("step"))
            .collect();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<discovery::Status>();
        app.init_resource::<starboard::canvas::Canvas>();
        app.init_resource::<starboard::input::OwnBrush>();
        app.insert_resource(testing::PendingAction::default());
        app.add_message::<starboard::input::PendingLocalDraw>();
        app.add_message::<discovery::Command>();
        app.insert_resource(StarboardScriptRunner::new(steps));
        app.add_systems(Update, testing::bevy_systems::run_script::<StarboardStep>);
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
            testing::ScriptVerdict::Failed
        );
    }

    #[test]
    fn a_join_step_arms_the_intent_with_the_room() {
        let mut app = app(&["join:room-1", "stay"]);
        app.update();
        let pending = app.world().resource::<testing::PendingAction>();
        assert!(matches!(pending.0, Some(testing::Action::Join(_))));
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
