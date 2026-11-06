use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::discovery;

use crate::ping;
use crate::ring::RingPeers;
use crate::shared::script::poll_shared_script_step::{SharedStepView, poll_shared_script_step};
use crate::shared::script::script_runner::{ScriptRunner, StepOutcome, app_exit_for_verdict};
use crate::shared::script::shared_script_step::SharedScriptStep;
use crate::shared::script::shared_step_effect_sink::SharedStepEffectSink;

pub type LobbyScriptRunner = ScriptRunner<SharedScriptStep>;

pub fn run_script(
    time: Res<Time>,
    status: Res<discovery::Status>,
    join_error: Option<Res<discovery::ui::JoinError>>,
    senders: Res<ping::RoomMessageSenders>,
    ring: Res<RingPeers>,
    mut runner: ResMut<LobbyScriptRunner>,
    mut effects: SharedStepEffectSink,
    mut exits: MessageWriter<AppExit>,
) {
    let view = SharedStepView {
        status: status.into_inner(),
        join_error: join_error
            .map(Res::into_inner)
            .and_then(|error| error.as_deref()),
        room_message_senders: senders.into_inner().peers.len(),
    };
    let ring_joined = ring.into_inner().get() > 0;
    let verdict = runner.advance(time.into_inner().delta(), |step, clock| {
        if matches!(step, SharedScriptStep::WaitRingJoined) {
            return ring_step_outcome(ring_joined);
        }
        let poll = poll_shared_script_step(step, clock, &view);
        if let Some(effect) = poll.effect {
            effects.apply(effect);
        }
        poll.outcome
    });
    if let Some(exit) = app_exit_for_verdict(verdict) {
        exits.write(exit);
    }
}

// needed helper: the live ring count decides `wait-ring-joined`; the deadline
// stays with the generic runner, so success is immediate and failure is a timeout
const fn ring_step_outcome(is_joined: bool) -> StepOutcome {
    if is_joined {
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

    use super::{LobbyScriptRunner, ring_step_outcome, run_script};
    use crate::ping::RoomMessageSenders;
    use crate::ring::RingPeers;
    use crate::shared::intent::PendingAction;
    use crate::shared::script::script_runner::StepOutcome;
    use crate::shared::script::shared_script_step::SharedScriptStep;
    use crate::shared::script::timed_script_step::parse_timed_script_step;

    fn app_with_script(script: &[&str], ring_peers: usize) -> App {
        let steps = script
            .iter()
            .map(|raw| parse_timed_script_step::<SharedScriptStep>(raw).expect("step"))
            .collect();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<discovery::Status>();
        app.init_resource::<RoomMessageSenders>();
        let ring = RingPeers::new();
        ring.set(ring_peers);
        app.insert_resource(ring);
        app.insert_resource(PendingAction::default());
        app.add_message::<discovery::Command>();
        app.insert_resource(LobbyScriptRunner::new(steps));
        app.add_systems(Update, run_script);
        app
    }

    #[test]
    fn test_usage() {
        let steps = ["create:room-1", "stay"]
            .iter()
            .map(|raw| parse_timed_script_step::<SharedScriptStep>(raw).expect("step"))
            .collect();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<discovery::Status>();
        app.init_resource::<RoomMessageSenders>();
        app.init_resource::<RingPeers>();
        app.insert_resource(PendingAction::default());
        app.add_message::<discovery::Command>();
        app.insert_resource(LobbyScriptRunner::new(steps));
        app.add_systems(Update, run_script);
        app.update();
        assert_eq!(app.world().resource::<LobbyScriptRunner>().index, 1);
        assert!(matches!(
            app.world().resource::<PendingAction>().0,
            Some(crate::shared::action::Action::Create(_))
        ));
    }

    #[test]
    fn ring_joined_succeeds_once_a_ring_peer_is_seen() {
        assert_eq!(ring_step_outcome(false), StepOutcome::Pending);
        assert_eq!(ring_step_outcome(true), StepOutcome::Done);
        let mut joined = app_with_script(&["wait-ring-joined@120s"], 1);
        joined.update();
        assert_eq!(joined.world().resource::<LobbyScriptRunner>().index, 1);
        let mut waiting = app_with_script(&["wait-ring-joined@120s"], 0);
        waiting.update();
        assert_eq!(waiting.world().resource::<LobbyScriptRunner>().index, 0);
    }

    #[test]
    fn ring_joined_fails_past_its_deadline_without_a_ring_peer() {
        use crate::shared::script::script_runner::ScriptVerdict;
        use crate::shared::script::timed_script_step::TimedScriptStep;
        let timed = TimedScriptStep {
            step: SharedScriptStep::WaitRingJoined,
            deadline: Some(Duration::from_secs(5)),
            raw: "wait-ring-joined@5s".to_string(),
        };
        let mut runner = LobbyScriptRunner::new(vec![timed]);
        let pending = |_: &SharedScriptStep, _| StepOutcome::Pending;
        assert_eq!(
            runner.advance(Duration::from_secs(5), pending),
            ScriptVerdict::Running
        );
        assert_eq!(
            runner.advance(Duration::from_secs(1), pending),
            ScriptVerdict::Failed
        );
    }
}
