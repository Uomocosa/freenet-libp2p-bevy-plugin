//! Step definitions for the starboard e2e scenarios.
//!
//! This crate is shared: `starboard` depends on it and adds
//! only the `PollableScriptStep` impl plus the conversion to the game's canvas
//! types; `tests/e2e` depends on it the same way for the test files. It uses no
//! game types so both crates can share it: `Cell` and `Stroke` are plain data,
//! and the script text spells the same verbs the app already parses
//! (`paint-from-to`, `erase-from-to`, `wait-canvas-nonempty`,
//! `wait-canvas-stable`, `wait-cell-painted`, plus every shared step).
//!
//! Rooms and deadlines never appear here: room steps use the `{room}` placeholder
//! that `Orchestrator` fills in per run, and waits carry no deadline until
//! `Orchestrator` assigns one per network (`.within()` overrides a single step).
//!
//! The step builders below are used by `tests/e2e`, not by the example binary
//! (which only needs the step type, its parsing, and the canvas conversion), so
//! unused-code warnings are silenced here like in `tests/common`.

#![allow(dead_code)]

use std::time::Duration;

use freenet_libp2p_bevy_plugin::identity::{PeerId, RoomName};
use freenet_libp2p_bevy_plugin::lobby_room::testing::LobbyStep;
use freenet_libp2p_bevy_plugin::testing::{
    E2EStep, MemberCount, ROOM_PLACEHOLDER, ScriptCheckpoint, StepWithDeadline,
    traits::{OrchestratedStep, ScriptStep},
};

/// One cell of the board, as `x.y` in script text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub x: u16,
    pub y: u16,
}

impl Cell {
    #[must_use]
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

/// One painted stroke, as `from-to` in script text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stroke {
    pub from: Cell,
    pub to: Cell,
}

impl Stroke {
    #[must_use]
    pub const fn new(from: Cell, to: Cell) -> Self {
        Self { from, to }
    }
}

/// One step of a starboard `--script`: a shared step, or a canvas step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StarboardStep {
    Lobby(LobbyStep),
    Shared(E2EStep),
    Paint(Vec<Stroke>),
    Erase(Vec<Stroke>),
    WaitCanvasNonEmpty,
    WaitCanvasStable,
    WaitCellPainted(Cell),
    AddFriend(String),
    Dial(String),
    WaitFriendConnected(PeerId),
    WaitFriendDisconnected(PeerId),
}

/// Mirrors `starboard::canvas::CANVAS_SIDE`; the game rejects cells at or above
/// it, so the script parser must reject them first with the same message.
const CANVAS_SIDE: u32 = 1024;

impl ScriptStep for StarboardStep {
    const STEP_NAMES: &'static str = "create, join, leave, wait-lobby-room, wait-lobby-room-gone, \
        wait-join-succeeded, wait-join-failed, wait-left, wait-members, \
        wait-members-are-strictly-less-than, wait-members-are-less-or-equal-to, wait-room-messages, \
        wait-ring-joined, wait-lobby-ready, sleep, mark-completed, exit-app, crash-app, stay, paint-from-to, \
        erase-from-to, wait-canvas-nonempty, wait-canvas-stable, wait-cell-painted, add-friend, dial, \
        wait-friend-connected, wait-friend-disconnected";

    fn parse_verb(verb: &str, arg: &str) -> Option<Result<Self, String>> {
        match verb {
            "paint-from-to" => Some(parse_stroke(arg).map(|stroke| Self::Paint(vec![stroke]))),
            "erase-from-to" => Some(parse_stroke(arg).map(|stroke| Self::Erase(vec![stroke]))),
            "wait-canvas-nonempty" => Some(Ok(Self::WaitCanvasNonEmpty)),
            "wait-canvas-stable" => Some(Ok(Self::WaitCanvasStable)),
            "wait-cell-painted" => Some(parse_cell(arg).map(Self::WaitCellPainted)),
            "add-friend" => Some(Ok(Self::AddFriend(arg.to_string()))),
            "dial" => Some(Ok(Self::Dial(arg.to_string()))),
            "wait-friend-connected" => Some(Ok(Self::WaitFriendConnected(PeerId::from(arg)))),
            "wait-friend-disconnected" => Some(Ok(Self::WaitFriendDisconnected(PeerId::from(arg)))),
            _ => LobbyStep::parse_verb(verb, arg)
                .map(|step| step.map(Self::Lobby))
                .or_else(|| E2EStep::parse_verb(verb, arg).map(|step| step.map(Self::Shared))),
        }
    }

    fn is_wait(&self) -> bool {
        match self {
            Self::Lobby(step) => step.is_wait(),
            Self::Shared(step) => step.is_wait(),
            Self::WaitCanvasNonEmpty
            | Self::WaitCanvasStable
            | Self::WaitCellPainted(_)
            | Self::WaitFriendConnected(_)
            | Self::WaitFriendDisconnected(_) => true,
            Self::Paint(_) | Self::Erase(_) | Self::AddFriend(_) | Self::Dial(_) => false,
        }
    }
}

impl OrchestratedStep for StarboardStep {
    fn script_text(&self) -> String {
        match self {
            Self::Lobby(step) => step.script_text(),
            Self::Shared(step) => step.script_text(),
            Self::Paint(strokes) => strokes
                .iter()
                .map(|stroke| {
                    format!(
                        "paint-from-to:{}.{}-{}.{}",
                        stroke.from.x, stroke.from.y, stroke.to.x, stroke.to.y
                    )
                })
                .collect::<Vec<String>>()
                .join(", "),
            Self::Erase(strokes) => strokes
                .iter()
                .map(|stroke| {
                    format!(
                        "erase-from-to:{}.{}-{}.{}",
                        stroke.from.x, stroke.from.y, stroke.to.x, stroke.to.y
                    )
                })
                .collect::<Vec<String>>()
                .join(", "),
            Self::WaitCanvasNonEmpty => "wait-canvas-nonempty".to_string(),
            Self::WaitCanvasStable => "wait-canvas-stable".to_string(),
            Self::WaitCellPainted(cell) => format!("wait-cell-painted:{}.{}", cell.x, cell.y),
            Self::AddFriend(peer) => format!("add-friend:{peer}"),
            Self::Dial(peer) => format!("dial:{peer}"),
            Self::WaitFriendConnected(id) => format!("wait-friend-connected:{id}"),
            Self::WaitFriendDisconnected(id) => format!("wait-friend-disconnected:{id}"),
        }
    }
}

/// Parses one `x.y` cell inside the canvas.
fn parse_cell(raw: &str) -> Result<Cell, String> {
    let Some((x, y)) = raw.split_once('.') else {
        return Err(format!("expected a cell like 10.20, got '{raw}'"));
    };
    if y.contains('.') {
        return Err(format!("expected a cell like 10.20, got '{raw}'"));
    }
    let (Ok(x), Ok(y)) = (x.parse::<u32>(), y.parse::<u32>()) else {
        return Err(format!("cell '{raw}' is not a number"));
    };
    if x >= CANVAS_SIDE || y >= CANVAS_SIDE {
        return Err(format!(
            "cell '{raw}' is outside the canvas (0..{CANVAS_SIDE})"
        ));
    }
    Ok(Cell {
        x: u16::try_from(x).unwrap_or(0),
        y: u16::try_from(y).unwrap_or(0),
    })
}

/// Parses `from-to` cells, e.g. `10.20-30.102`, into a stroke on the board.
fn parse_stroke(raw: &str) -> Result<Stroke, String> {
    let Some((from, to)) = raw.split_once('-') else {
        return Err(format!("expected a stroke like 10.20-30.102, got '{raw}'"));
    };
    if to.contains('-') {
        return Err(format!("expected a stroke like 10.20-30.102, got '{raw}'"));
    }
    Ok(Stroke {
        from: parse_cell(from)?,
        to: parse_cell(to)?,
    })
}

/// Creates the run's room and waits until the join succeeded (the `{room}`
/// placeholder is filled per run; the wait keeps the old 180 s mainnet budget).
#[must_use]
pub fn create_room() -> Vec<StepWithDeadline<StarboardStep>> {
    vec![
        StepWithDeadline::from(StarboardStep::Lobby(LobbyStep::CreateRoom(RoomName::from(
            ROOM_PLACEHOLDER,
        )))),
        StepWithDeadline::from(StarboardStep::Lobby(LobbyStep::WaitJoinSucceeded(
            RoomName::from(ROOM_PLACEHOLDER),
        )))
        .within(Duration::from_secs(180)),
    ]
}

/// Joins the run's room: waits for the room in the lobby, joins it, waits
/// until the join succeeded (the old 420 s mainnet budgets on both waits).
#[must_use]
pub fn join_room() -> Vec<StepWithDeadline<StarboardStep>> {
    vec![
        StepWithDeadline::from(StarboardStep::Lobby(LobbyStep::WaitLobbyRoom(
            RoomName::from(ROOM_PLACEHOLDER),
        )))
        .within(Duration::from_secs(420)),
        StepWithDeadline::from(StarboardStep::Lobby(LobbyStep::JoinRoom(RoomName::from(
            ROOM_PLACEHOLDER,
        )))),
        StepWithDeadline::from(StarboardStep::Lobby(LobbyStep::WaitJoinSucceeded(
            RoomName::from(ROOM_PLACEHOLDER),
        )))
        .within(Duration::from_secs(420)),
    ]
}

/// Waits until the room (including this peer) has `count` members.
#[must_use]
pub fn wait_until_room_has_members(count: u32) -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::Shared(
        E2EStep::WaitMembers(MemberCount(count)),
    ))]
}

/// Waits until the room has fewer members than `count` (strictly less).
#[must_use]
pub fn wait_until_room_has_fewer_members_than(count: u32) -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::Shared(
        E2EStep::WaitMembersAreStrictlyLessThan(MemberCount(count)),
    ))]
}

/// Waits until the room has at most `count` members (less or equal).
#[must_use]
pub fn wait_until_room_has_at_most_members(count: u32) -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::Shared(
        E2EStep::WaitMembersAreLessOrEqualTo(MemberCount(count)),
    ))]
}

/// Waits until the run's room shows up in the lobby.
#[must_use]
pub fn wait_until_lobby_has_room() -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::Lobby(
        LobbyStep::WaitLobbyRoom(RoomName::from(ROOM_PLACEHOLDER)),
    ))]
}

/// Waits until the run's room is gone from the lobby (all holders quit or crashed
/// and the lease lapsed). The watcher uses it so the late joiner only starts
/// after the room is really gone.
#[must_use]
pub fn wait_until_room_is_gone() -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::Lobby(
        LobbyStep::WaitLobbyRoomGone(RoomName::from(ROOM_PLACEHOLDER)),
    ))]
}

/// Leaves the run's room.
#[must_use]
pub fn leave_room() -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::Lobby(
        LobbyStep::LeaveRoom(RoomName::from(ROOM_PLACEHOLDER)),
    ))]
}

/// Waits until this peer left the run's room.
#[must_use]
pub fn wait_until_left_room() -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::Lobby(
        LobbyStep::WaitLeft(RoomName::from(ROOM_PLACEHOLDER)),
    ))]
}

/// Waits until the join of the run's room failed (room gone or bad token).
#[must_use]
pub fn wait_until_join_fails() -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::Lobby(
        LobbyStep::WaitJoinFailed(RoomName::from(ROOM_PLACEHOLDER)),
    ))]
}

/// Waits until the join of `room` failed (for a room name that is not the run's room).
#[must_use]
pub fn wait_until_join_fails_named(room: &str) -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::Lobby(
        LobbyStep::WaitJoinFailed(RoomName::from(room)),
    ))]
}

/// Joins `room` (a pattern that may contain the `{room}` placeholder).
#[must_use]
pub fn join_room_named(room: &str) -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::Lobby(
        LobbyStep::JoinRoom(RoomName::from(room)),
    ))]
}

/// Waits until `cell` is painted.
#[must_use]
pub fn wait_until_cell_is_painted(cell: Cell) -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::WaitCellPainted(cell))]
}

/// Sleeps `secs` seconds; the orchestrator scales it by the run's time scale on fake.
/// Kept only when the room itself cannot be the sync point (S4a history write, S3 rejoin gap).
#[must_use]
pub fn sleep_for_secs(secs: u64) -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::Shared(
        E2EStep::Sleep(Duration::from_secs(secs)),
    ))]
}

/// Aborts the process at once, like a crash (no quit published).
#[must_use]
pub fn crash_app() -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::Shared(
        E2EStep::CrashApp,
    ))]
}

/// Stays alive until the run ends (killed by the harness).
#[must_use]
pub fn stay() -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::Shared(E2EStep::Stay))]
}

/// Paints every stroke in order.
#[must_use]
pub fn paint(strokes: &[Stroke]) -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::Paint(
        strokes.to_vec(),
    ))]
}

/// Erases every stroke in order.
#[must_use]
pub fn erase(strokes: &[Stroke]) -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::Erase(
        strokes.to_vec(),
    ))]
}

/// Waits until any canvas tile is painted.
#[must_use]
pub fn wait_until_canvas_has_content() -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::WaitCanvasNonEmpty)]
}

/// Waits until the canvas hash holds still.
#[must_use]
pub fn wait_until_canvas_stops_changing() -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::WaitCanvasStable)]
}

/// Completes `checkpoint` so peers waiting on it can start.
#[must_use]
pub fn mark_completed(checkpoint: ScriptCheckpoint) -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::Shared(
        E2EStep::MarkCompleted(checkpoint),
    ))]
}

/// Quits the app once the script is done.
#[must_use]
pub fn exit_app() -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::Lobby(
        LobbyStep::ExitApp,
    ))]
}

/// Types one peer address into the Friends tab and adds it.
///
/// The orchestrator replaces the peer name with its live addresses at spawn,
/// so by the time the app runs every step carries a full multiaddr.
#[must_use]
pub fn add_friend(peer: &str) -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::AddFriend(
        peer.to_string(),
    ))]
}

/// Dials one of the peer's addresses once, without adding a friend: the
/// orchestrator replaces the peer name with its live addresses at spawn.
#[must_use]
pub fn dial(peer: &str) -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::Dial(
        peer.to_string(),
    ))]
}

/// Waits until the friend row for the peer shows connected: the orchestrator
/// replaces the peer name with its stable id at spawn.
#[must_use]
pub fn wait_until_friend_is_connected(peer: &str) -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(StarboardStep::WaitFriendConnected(
        PeerId::from(peer),
    ))]
}

/// Waits until the friend row for the peer shows offline: the orchestrator
/// replaces the peer name with its stable id at spawn.
#[must_use]
pub fn wait_until_friend_is_disconnected(peer: &str) -> Vec<StepWithDeadline<StarboardStep>> {
    vec![StepWithDeadline::from(
        StarboardStep::WaitFriendDisconnected(PeerId::from(peer)),
    )]
}

#[cfg(test)]
mod tests {
    use freenet_libp2p_bevy_plugin::testing::{
        ScriptCheckpoint, StepWithDeadline, TimedScriptStep,
        traits::{OrchestratedStep, ScriptStep},
    };

    use super::{
        Cell, StarboardStep, Stroke, add_friend, create_room, dial, erase, exit_app, join_room,
        mark_completed, paint, wait_until_canvas_has_content, wait_until_canvas_stops_changing,
        wait_until_friend_is_connected, wait_until_room_has_members,
    };

    fn step(raw: &str) -> Result<StarboardStep, String> {
        TimedScriptStep::<StarboardStep>::try_from(raw).map(|timed| timed.step)
    }

    /// Renders one step group the way the orchestrator writes it into the
    /// peer's `--script`: the step text, plus `@Ns` for a wait carrying its
    /// own deadline.
    fn rendered(group: &[StepWithDeadline<StarboardStep>]) -> String {
        group
            .iter()
            .map(|pending| {
                pending.within.map_or_else(
                    || pending.step.script_text(),
                    |within| format!("{}@{}s", pending.step.script_text(), within.as_secs()),
                )
            })
            .collect::<Vec<String>>()
            .join(", ")
    }

    /// Every verb the peer runs, in order: one group per builder, one
    /// comma-separated fragment per step, the verb before any `:` argument.
    fn verbs(groups: &[Vec<StepWithDeadline<StarboardStep>>]) -> Vec<String> {
        let scripts: Vec<String> = groups.iter().map(|group| rendered(group)).collect();
        scripts
            .iter()
            .flat_map(|script| script.split(", "))
            .map(|fragment| {
                fragment
                    .split([':', '@'])
                    .next()
                    .unwrap_or_default()
                    .to_string()
            })
            .collect()
    }

    #[test]
    fn test_usage() {
        assert_eq!(
            step("paint-from-to:10.20-30.102"),
            Ok(StarboardStep::Paint(vec![Stroke::new(
                Cell::new(10, 20),
                Cell::new(30, 102)
            )]))
        );
        assert_eq!(
            step("wait-canvas-stable@30s"),
            Ok(StarboardStep::WaitCanvasStable)
        );
        assert_eq!(
            step("mark-completed:painting-done"),
            Ok(StarboardStep::Shared(
                freenet_libp2p_bevy_plugin::testing::E2EStep::MarkCompleted(ScriptCheckpoint::new(
                    "painting-done"
                ))
            ))
        );
    }

    #[test]
    fn step_functions_spell_valid_script_fragments() {
        let strokes = [
            Stroke::new(Cell::new(200, 300), Cell::new(260, 300)),
            Stroke::new(Cell::new(230, 309), Cell::new(290, 309)),
        ];
        assert_eq!(
            rendered(&paint(&strokes)),
            "paint-from-to:200.300-260.300, paint-from-to:230.309-290.309"
        );
        assert_eq!(
            rendered(&erase(&strokes[..1])),
            "erase-from-to:200.300-260.300"
        );
        assert_eq!(rendered(&wait_until_room_has_members(3)), "wait-members:3");
        assert_eq!(
            rendered(&wait_until_canvas_has_content()),
            "wait-canvas-nonempty"
        );
        assert_eq!(
            rendered(&wait_until_canvas_stops_changing()),
            "wait-canvas-stable"
        );
        assert_eq!(
            rendered(&mark_completed(ScriptCheckpoint::new("done"))),
            "mark-completed:done"
        );
        assert_eq!(rendered(&exit_app()), "exit-app");
    }

    #[test]
    fn room_groups_spell_the_old_openings() {
        assert_eq!(
            rendered(&create_room()),
            "create:{room}, wait-join-succeeded:{room}@180s"
        );
        assert_eq!(
            rendered(&join_room()),
            "wait-lobby-room:{room}@420s, join:{room}, wait-join-succeeded:{room}@420s"
        );
    }

    #[test]
    fn room_groups_carry_their_own_deadlines() {
        for pending in create_room().iter().chain(join_room().iter()) {
            let is_wait = pending.step.is_wait();
            assert_eq!(pending.within.is_some(), is_wait);
        }
        let waits: Vec<u64> = create_room()
            .iter()
            .chain(join_room().iter())
            .filter(|pending| pending.step.is_wait())
            .map(|pending| pending.within.map_or(0, |within| within.as_secs()))
            .collect();
        assert_eq!(waits, vec![180, 420, 420]);
    }

    /// S1's helper runs the same steps, in the same order, as the old
    /// `Scenario::starboard` joiner (`tests/common/scenario.rs`): the lobby
    /// wait, the join, the join wait, the member wait, the canvas wait, the
    /// three erases, the stable wait and the exit — ignoring the old script's
    /// final `sleep:10s` before `exit-app`.
    #[test]
    fn s1_helper_matches_the_old_joiner_order() {
        let erases = [
            Stroke::new(Cell::new(200, 300), Cell::new(260, 300)),
            Stroke::new(Cell::new(230, 309), Cell::new(290, 309)),
            Stroke::new(Cell::new(260, 318), Cell::new(320, 318)),
        ];
        let helper = [
            join_room(),
            wait_until_room_has_members(3),
            wait_until_canvas_has_content(),
            erase(&erases),
            wait_until_canvas_stops_changing(),
            exit_app(),
        ];
        assert_eq!(
            verbs(&helper),
            vec![
                "wait-lobby-room",
                "join",
                "wait-join-succeeded",
                "wait-members",
                "wait-canvas-nonempty",
                "erase-from-to",
                "erase-from-to",
                "erase-from-to",
                "wait-canvas-stable",
                "exit-app",
            ]
        );
    }

    #[test]
    fn canvas_waits_need_a_deadline() {
        assert!(step("wait-canvas-nonempty").is_err());
        assert!(step("wait-cell-painted:10.20").is_err());
    }

    #[test]
    fn friend_steps_round_trip_through_text() {
        assert_eq!(
            step("add-friend:bob"),
            Ok(StarboardStep::AddFriend("bob".to_string()))
        );
        assert_eq!(step("dial:bob"), Ok(StarboardStep::Dial("bob".to_string())));
        assert_eq!(
            step("wait-friend-connected:peer-1@30s"),
            Ok(StarboardStep::WaitFriendConnected(
                freenet_libp2p_bevy_plugin::identity::PeerId::from("peer-1")
            ))
        );
        assert!(step("wait-friend-connected:peer-1").is_err());
        assert!(step("add-friend:bob@30s").is_err());
    }

    #[test]
    fn friend_builders_spell_their_steps() {
        assert_eq!(rendered(&add_friend("bob")), "add-friend:bob");
        assert_eq!(rendered(&dial("bob")), "dial:bob");
        assert_eq!(
            rendered(&wait_until_friend_is_connected("bob")),
            "wait-friend-connected:bob"
        );
    }

    #[test]
    fn a_bad_stroke_is_rejected() {
        assert!(step("paint-from-to:10.20").is_err());
        assert!(step("paint-from-to:10.20-9999.5").is_err());
    }

    /// The harness writes the script, this parser reads it. The same shape is
    /// pinned on the harness side (`tests/common/scenario.rs`).
    #[test]
    fn the_harness_script_shape_parses() {
        let script = "create:room-1, wait-join-succeeded:room-1@120s, \
                      paint-from-to:200.300-260.300, wait-canvas-stable@30s, sleep:45s, \
                      leave:room-1, wait-left:room-1@30s, exit-app";
        let steps: Result<Vec<StarboardStep>, String> = script.split(", ").map(step).collect();
        assert_eq!(steps.map(|steps| steps.len()), Ok(8));
    }

    #[test]
    fn every_lobby_verb_the_scripts_use_parses_to_the_same_text() {
        let lines = [
            ("create", "{room}"),
            ("create", "room-1"),
            ("join", "{room}"),
            ("join", "r"),
            ("join", "no-such-room-{room}"),
            ("leave", "{room}"),
            ("wait-lobby-room", "{room}"),
            ("wait-lobby-room-gone", "{room}"),
            ("wait-join-succeeded", "{room}"),
            ("wait-join-failed", "{room}"),
            ("wait-left", "{room}"),
            ("wait-ring-joined", ""),
            ("wait-lobby-ready", ""),
            ("exit-app", ""),
        ];
        for (verb, arg) in lines {
            let step = StarboardStep::parse_verb(verb, arg)
                .expect("known verb")
                .expect("parses");
            assert!(
                matches!(step, StarboardStep::Lobby(_)),
                "{verb} must stay a lobby step"
            );
            let text = if arg.is_empty() {
                verb.to_string()
            } else {
                format!("{verb}:{arg}")
            };
            assert_eq!(step.script_text(), text);
        }
    }
}
