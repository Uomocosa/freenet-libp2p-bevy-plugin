use std::time::Duration;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use discovery::state_machine::DiscoveryState;
use e2e_steps::starboard_steps::StarboardStep;
use freenet_libp2p_bevy_plugin::{discovery, identity, p2p, peers, testing, ui};

use crate::friend_intent::FriendIntent;
use crate::scripted_starboard_step::ScriptedStarboardStep;
use crate::starboard;

/// How long the canvas must keep one hash before `wait-canvas-stable` is done.
const CANVAS_STABLE_FOR: Duration = Duration::from_secs(3);

/// The canvas hash `wait-canvas-stable` is watching, and since when it has held.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanvasHashSince {
    pub hash: u64,
    pub since: Duration,
}

pub type StarboardScriptRunner = testing::ScriptRunner<ScriptedStarboardStep>;

#[derive(SystemParam)]
pub struct StarboardStepParam<'w, 's> {
    state: Option<Res<'w, DiscoveryState>>,
    join_error: Option<Res<'w, ui::discovery::JoinError>>,
    canvas: Res<'w, starboard::canvas::Canvas>,
    brush: Res<'w, starboard::input::OwnBrush>,
    draws: MessageWriter<'w, starboard::input::PendingLocalDraw>,
    effects: testing::E2EStepEffectSink<'w>,
    stable: Local<'s, Option<CanvasHashSince>>,
    friend_intent: ResMut<'w, FriendIntent>,
    pending_dials: ResMut<'w, p2p::PendingLibp2pCommands>,
    connected_peers: Query<'w, 's, &'static peers::Peer, With<peers::Connected>>,
}

impl testing::traits::PollableScriptStep for ScriptedStarboardStep {
    type Param<'w, 's> = StarboardStepParam<'w, 's>;

    fn poll(
        step: &Self,
        clock: testing::StepClock,
        param: &mut Self::Param<'_, '_>,
    ) -> testing::StepOutcome {
        let fallback = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me")).build();
        let state: &DiscoveryState = param.state.as_deref().unwrap_or(&fallback);
        match &step.0 {
            StarboardStep::Shared(shared) => {
                let view = testing::E2EStepView {
                    status: state,
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
            StarboardStep::Paint(strokes) => {
                for stroke in strokes {
                    let action = starboard::canvas::DrawAction::Paint(param.brush.color);
                    let outcome = draw_stroke(
                        action,
                        crate::script::to_canvas_stroke(*stroke),
                        state,
                        *param.brush,
                        &mut param.draws,
                    );
                    if outcome != testing::StepOutcome::Done {
                        return outcome;
                    }
                }
                testing::StepOutcome::Done
            }
            StarboardStep::Erase(strokes) => {
                for stroke in strokes {
                    let outcome = draw_stroke(
                        starboard::canvas::DrawAction::Erase,
                        crate::script::to_canvas_stroke(*stroke),
                        state,
                        *param.brush,
                        &mut param.draws,
                    );
                    if outcome != testing::StepOutcome::Done {
                        return outcome;
                    }
                }
                testing::StepOutcome::Done
            }
            StarboardStep::WaitCanvasNonEmpty => {
                if param.canvas.tiles.iter().any(Option::is_some) {
                    testing::StepOutcome::Done
                } else {
                    testing::StepOutcome::Pending
                }
            }
            StarboardStep::WaitCanvasStable => is_canvas_stable(
                &mut param.stable,
                starboard::canvas::hash_canvas(&param.canvas),
                clock,
            ),
            StarboardStep::WaitCellPainted(pos) => {
                if starboard::canvas::read_cell(&param.canvas, crate::script::to_canvas_cell(*pos))
                    .color
                    .is_some()
                {
                    testing::StepOutcome::Done
                } else {
                    testing::StepOutcome::Pending
                }
            }
            StarboardStep::AddFriend(addr) => poll_add_friend(addr, clock, param),
            StarboardStep::Dial(addrs) => poll_dial(addrs, param),
            StarboardStep::WaitFriendConnected(id) => {
                if param.connected_peers.iter().any(|peer| *peer == *id) {
                    testing::StepOutcome::Done
                } else {
                    testing::StepOutcome::Pending
                }
            }
            StarboardStep::WaitFriendDisconnected(id) => {
                if param.connected_peers.iter().any(|peer| *peer == *id) {
                    testing::StepOutcome::Pending
                } else {
                    testing::StepOutcome::Done
                }
            }
        }
    }
}

// needed helper: arms the friend intent for one address, failing fast on bad
// addresses and when the tab never accepts the address
fn poll_add_friend(
    addr: &str,
    clock: testing::StepClock,
    param: &mut StarboardStepParam<'_, '_>,
) -> testing::StepOutcome {
    if param.friend_intent.is_done(addr) {
        return testing::StepOutcome::Done;
    }
    if identity::Peer::try_from(addr).is_err() {
        return testing::StepOutcome::Failed(format!("bad friend address: {addr}"));
    }
    if clock.elapsed >= Duration::from_secs(180) {
        return testing::StepOutcome::Failed(format!("adding {addr} timed out"));
    }
    param.friend_intent.arm(addr);
    testing::StepOutcome::Pending
}

// needed helper: pushes one one-off dial command without adding a friend
fn poll_dial(addrs: &str, param: &mut StarboardStepParam<'_, '_>) -> testing::StepOutcome {
    match dial_peer(addrs) {
        Ok(peer) => {
            param.pending_dials.push(p2p::Libp2pCommand::Dial(peer));
            testing::StepOutcome::Done
        }
        Err(reason) => testing::StepOutcome::Failed(reason),
    }
}

// needed helper: draws one scripted stroke; drawing outside a room is a script error
fn draw_stroke(
    action: starboard::canvas::DrawAction,
    stroke: starboard::canvas::Stroke,
    state: &DiscoveryState,
    brush: starboard::input::OwnBrush,
    draws: &mut MessageWriter<starboard::input::PendingLocalDraw>,
) -> testing::StepOutcome {
    if !starboard::room::is_in_room(state) {
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

// needed helper: parses one `dial` step into a single one-off dial command
fn dial_peer(addrs: &str) -> Result<identity::Peer, String> {
    let mut parsed = Vec::new();
    for addr in addrs.split_whitespace() {
        match identity::Peer::try_from(addr) {
            Ok(peer) => parsed.push(peer),
            Err(error) => return Err(format!("bad dial address {addr}: {error}")),
        }
    }
    let Some(first) = parsed.first() else {
        return Err("dial needs an address".to_string());
    };
    Ok(identity::Peer {
        id: first.id.clone(),
        addrs: parsed.iter().flat_map(|peer| peer.addrs.clone()).collect(),
    })
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{discovery, identity, testing};

    use super::StarboardScriptRunner;
    use super::dial_peer;
    use crate::scripted_starboard_step::ScriptedStarboardStep;
    use crate::starboard;

    fn app(script: &[&str]) -> App {
        let steps = script
            .iter()
            .map(|raw| {
                testing::TimedScriptStep::<ScriptedStarboardStep>::try_from(*raw).expect("step")
            })
            .collect();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<starboard::canvas::Canvas>();
        app.init_resource::<starboard::input::OwnBrush>();
        app.insert_resource(testing::PendingAction::default());
        app.insert_resource(crate::friend_intent::FriendIntent::default());
        app.insert_resource(freenet_libp2p_bevy_plugin::p2p::PendingLibp2pCommands::default());
        app.add_message::<starboard::input::PendingLocalDraw>();
        app.add_message::<discovery::Command>();
        app.insert_resource(StarboardScriptRunner::new(steps));
        app.add_systems(
            Update,
            testing::bevy_systems::run_script::<ScriptedStarboardStep>,
        );
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

    #[test]
    fn dial_merges_every_address_into_one_peer() {
        let id = libp2p::PeerId::random();
        let text =
            format!("/ip4/127.0.0.1/tcp/4001/p2p/{id} /ip4/127.0.0.1/udp/4002/quic-v1/p2p/{id}");
        let peer = dial_peer(&text).expect("dial");
        assert_eq!(peer.id, identity::PeerId::from(id.to_string()));
        assert_eq!(peer.addrs.len(), 2);
        assert!(dial_peer("not a multiaddr").is_err());
        assert!(dial_peer("   ").is_err());
    }

    #[test]
    fn a_dial_step_pushes_a_libp2p_dial_command() {
        let id = libp2p::PeerId::random();
        let raw = format!("dial:/ip4/127.0.0.1/tcp/4001/p2p/{id}");
        let mut app = app(&[raw.as_str()]);
        app.update();
        assert_eq!(app.world().resource::<StarboardScriptRunner>().index, 1);
        let pending = app
            .world()
            .resource::<freenet_libp2p_bevy_plugin::p2p::PendingLibp2pCommands>();
        assert_eq!(pending.len(), 1);
    }

    #[test]
    fn a_friend_wait_is_done_once_the_peer_connects() {
        let raw = "wait-friend-connected:peer-1@30s";
        let mut app = app(&[raw]);
        app.update();
        assert_eq!(app.world().resource::<StarboardScriptRunner>().index, 0);
        app.world_mut().spawn((
            freenet_libp2p_bevy_plugin::peers::Peer::from(identity::PeerId::from("peer-1")),
            freenet_libp2p_bevy_plugin::peers::Connected,
        ));
        app.update();
        assert_eq!(app.world().resource::<StarboardScriptRunner>().index, 1);
    }
}
