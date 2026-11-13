use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use discovery::state_machine::DiscoveryState;
use freenet_libp2p_bevy_plugin::identity;
use freenet_libp2p_bevy_plugin::testing::traits::{PollableScriptStep, ScriptStep};
use freenet_libp2p_bevy_plugin::{discovery, testing, ui};

use crate::ping;
use crate::ring::RingPeers;

pub type LobbyScriptRunner = testing::ScriptRunner<LobbyStep>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LobbyStep {
    Lobby(testing::LobbyStep),
    Shared(testing::E2EStep),
}

impl ScriptStep for LobbyStep {
    const STEP_NAMES: &'static str = "create, join, leave, wait-lobby-room, wait-lobby-room-gone, \
        wait-join-succeeded, wait-join-failed, wait-left, wait-members, \
        wait-members-are-strictly-less-than, wait-members-are-less-or-equal-to, wait-room-messages, \
        wait-ring-joined, wait-lobby-ready, sleep, mark-completed, exit-app, crash-app, stay";

    fn parse_verb(verb: &str, arg: &str) -> Option<Result<Self, String>> {
        testing::LobbyStep::parse_verb(verb, arg)
            .map(|step| step.map(Self::Lobby))
            .or_else(|| testing::E2EStep::parse_verb(verb, arg).map(|step| step.map(Self::Shared)))
    }

    fn is_wait(&self) -> bool {
        match self {
            Self::Lobby(step) => step.is_wait(),
            Self::Shared(step) => step.is_wait(),
        }
    }
}

#[derive(SystemParam)]
pub struct LobbyStepParam<'w> {
    state: Option<Res<'w, DiscoveryState>>,
    join_error: Option<Res<'w, ui::discovery::JoinError>>,
    senders: Res<'w, ping::RoomMessageSenders>,
    ring: Res<'w, RingPeers>,
    effects: testing::E2EStepEffectSink<'w>,
}

impl PollableScriptStep for LobbyStep {
    type Param<'w, 's> = LobbyStepParam<'w>;

    fn poll(
        step: &Self,
        clock: testing::StepClock,
        param: &mut Self::Param<'_, '_>,
    ) -> testing::StepOutcome {
        if matches!(step, Self::Lobby(testing::LobbyStep::WaitRingJoined)) {
            return ring_step_outcome(param.ring.get() > 0);
        }
        let fallback = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me")).build();
        let view_status = param.state.as_deref().unwrap_or(&fallback);
        let poll = match step {
            Self::Lobby(lobby) => {
                let view = testing::LobbyStepView {
                    status: view_status,
                    join_error: param
                        .join_error
                        .as_deref()
                        .and_then(|error| error.as_deref()),
                };
                lobby.poll_lobby_step(clock, &view)
            }
            Self::Shared(shared) => {
                let view = testing::E2EStepView {
                    room_message_senders: param.senders.peers.len(),
                    members: view_status.connected_members().count().saturating_add(1),
                };
                shared.poll_shared_script_step(clock, &view)
            }
        };
        if let Some(effect) = poll.effect {
            param.effects.apply(effect);
        }
        poll.outcome
    }
}

// needed helper: the live ring count decides `wait-ring-joined`; the deadline
// stays with the generic runner, so success is immediate and failure is a timeout
const fn ring_step_outcome(is_joined: bool) -> testing::StepOutcome {
    if is_joined {
        testing::StepOutcome::Done
    } else {
        testing::StepOutcome::Pending
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::testing;

    use super::{LobbyScriptRunner, LobbyStep, ring_step_outcome};
    use crate::ping::RoomMessageSenders;
    use crate::ring::RingPeers;

    fn app_with_script(script: &[&str], ring_peers: usize) -> App {
        let steps = script
            .iter()
            .map(|raw| testing::TimedScriptStep::<LobbyStep>::try_from(*raw).expect("step"))
            .collect();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<RoomMessageSenders>();
        let ring = RingPeers::new();
        ring.set(ring_peers);
        app.insert_resource(ring);
        app.insert_resource(testing::PendingAction::default());
        app.add_message::<testing::E2EExtensionEffect>();
        app.insert_resource(LobbyScriptRunner::new(steps));
        app.add_systems(Update, testing::bevy_systems::run_script::<LobbyStep>);
        app
    }

    #[test]
    fn test_usage() {
        let steps = ["create:room-1", "stay"]
            .iter()
            .map(|raw| testing::TimedScriptStep::<LobbyStep>::try_from(*raw).expect("step"))
            .collect();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<RoomMessageSenders>();
        app.init_resource::<RingPeers>();
        app.insert_resource(testing::PendingAction::default());
        app.add_message::<testing::E2EExtensionEffect>();
        app.insert_resource(LobbyScriptRunner::new(steps));
        app.add_systems(Update, testing::bevy_systems::run_script::<LobbyStep>);
        app.update();
        assert_eq!(app.world().resource::<LobbyScriptRunner>().index, 1);
        assert!(matches!(
            app.world().resource::<testing::PendingAction>().0,
            Some(testing::Action::Create(_))
        ));
    }

    #[test]
    fn ring_joined_succeeds_once_a_ring_peer_is_seen() {
        assert_eq!(ring_step_outcome(false), testing::StepOutcome::Pending);
        assert_eq!(ring_step_outcome(true), testing::StepOutcome::Done);
        let mut joined = app_with_script(&["wait-ring-joined@120s"], 1);
        joined.update();
        assert_eq!(joined.world().resource::<LobbyScriptRunner>().index, 1);
        let mut waiting = app_with_script(&["wait-ring-joined@120s"], 0);
        waiting.update();
        assert_eq!(waiting.world().resource::<LobbyScriptRunner>().index, 0);
    }

    #[test]
    fn ring_joined_fails_past_its_deadline_without_a_ring_peer() {
        use freenet_libp2p_bevy_plugin::testing::ScriptVerdict;
        let timed = testing::TimedScriptStep {
            step: LobbyStep::Lobby(testing::LobbyStep::WaitRingJoined),
            deadline: Some(Duration::from_secs(5)),
            raw: "wait-ring-joined@5s".to_string(),
        };
        let mut runner = LobbyScriptRunner::new(vec![timed]);
        let pending = |_: &LobbyStep, _| testing::StepOutcome::Pending;
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
