use std::time::Duration;

use derive_more::{Deref, From};
use freenet_libp2p_bevy_plugin::identity;

use crate::parse_stroke::parse_stroke;
use crate::starboard;

/// One step of a scripted session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptStep {
    CreateRoom(identity::RoomName),
    JoinRoom(identity::RoomName),
    LeaveRoom(identity::RoomName),
    WaitForNMembers(MemberCount),
    WaitSecs(Duration),
    WaitConverged(identity::RoomName),
    PaintFromTo(starboard::canvas::Stroke),
    EraseFromTo(starboard::canvas::Stroke),
    ExitGame,
}

impl ScriptStep {
    /// The step's name, as the script log reports it.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::CreateRoom(_) => "create",
            Self::JoinRoom(_) => "join",
            Self::LeaveRoom(_) => "leave",
            Self::WaitForNMembers(_) => "wait-members",
            Self::WaitSecs(_) => "wait-secs",
            Self::WaitConverged(_) => "wait-converged",
            Self::PaintFromTo(_) => "paint",
            Self::EraseFromTo(_) => "erase",
            Self::ExitGame => "exit",
        }
    }
}

/// How many room members a step waits for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deref, From)]
pub struct MemberCount(pub u32);

const STEP_NAMES: &str = "create, join, leave, wait-for-n-members, wait-secs, wait-converged, \
                          paint-from-to, erase-from-to, exit";

/// Parses one step of the `--script` list, e.g. `paint-from-to:10.20-30.102`.
///
/// # Errors
///
/// Returns a message naming the offending part when the verb is unknown, an
/// argument is missing, or an argument does not parse.
pub fn parse_script_step(raw: &str) -> Result<ScriptStep, String> {
    let text = raw.trim();
    let (verb, arg) = text.split_once(':').unwrap_or((text, ""));
    match verb {
        "create" => Ok(ScriptStep::CreateRoom(room_name(verb, arg)?)),
        "join" => Ok(ScriptStep::JoinRoom(room_name(verb, arg)?)),
        "leave" => Ok(ScriptStep::LeaveRoom(room_name(verb, arg)?)),
        "wait-for-n-members" => Ok(ScriptStep::WaitForNMembers(MemberCount(count(
            verb,
            arg,
            "a member count",
        )?))),
        "wait-secs" => Ok(ScriptStep::WaitSecs(Duration::from_secs(u64::from(count(
            verb,
            arg,
            "a number of seconds",
        )?)))),
        "wait-converged" => Ok(ScriptStep::WaitConverged(room_name(verb, arg)?)),
        "paint-from-to" => Ok(ScriptStep::PaintFromTo(parse_stroke(arg)?)),
        "erase-from-to" => Ok(ScriptStep::EraseFromTo(parse_stroke(arg)?)),
        "exit" => Ok(ScriptStep::ExitGame),
        other => Err(format!(
            "unknown step '{other}'; expected one of {STEP_NAMES}"
        )),
    }
}

// needed helper: a step's room name argument, which must not be empty
fn room_name(verb: &str, arg: &str) -> Result<identity::RoomName, String> {
    let name = arg.trim();
    if name.is_empty() {
        return Err(format!("step '{verb}' needs a room name"));
    }
    Ok(identity::RoomName::from(name))
}

// needed helper: a step's numeric argument, which must be a count
fn count(verb: &str, arg: &str, what: &str) -> Result<u32, String> {
    arg.trim()
        .parse::<u32>()
        .map_err(|_| format!("step '{verb}' needs {what}, got '{arg}'"))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use freenet_libp2p_bevy_plugin::identity;

    use super::{MemberCount, ScriptStep, parse_script_step};
    use crate::starboard;

    #[test]
    fn test_usage() {
        let room = identity::RoomName::from("room-1");
        assert_eq!(
            parse_script_step("create:room-1").expect("create"),
            ScriptStep::CreateRoom(room.clone())
        );
        assert_eq!(
            parse_script_step("join:room-1").expect("join"),
            ScriptStep::JoinRoom(room.clone())
        );
        assert_eq!(
            parse_script_step("leave:room-1").expect("leave"),
            ScriptStep::LeaveRoom(room.clone())
        );
        assert_eq!(
            parse_script_step("wait-for-n-members:2").expect("members"),
            ScriptStep::WaitForNMembers(MemberCount(2))
        );
        assert_eq!(
            parse_script_step("wait-secs:20").expect("secs"),
            ScriptStep::WaitSecs(Duration::from_secs(20))
        );
        assert_eq!(
            parse_script_step("wait-converged:room-1").expect("converged"),
            ScriptStep::WaitConverged(room.clone())
        );
        assert_eq!(
            parse_script_step("paint-from-to:10.20-30.102").expect("paint"),
            ScriptStep::PaintFromTo(starboard::canvas::Stroke {
                from: starboard::canvas::CellPos { x: 10, y: 20 },
                to: starboard::canvas::CellPos { x: 30, y: 102 },
            })
        );
        assert_eq!(
            parse_script_step("erase-from-to:10.20-30.102").expect("erase"),
            ScriptStep::EraseFromTo(starboard::canvas::Stroke {
                from: starboard::canvas::CellPos { x: 10, y: 20 },
                to: starboard::canvas::CellPos { x: 30, y: 102 },
            })
        );
        assert_eq!(
            parse_script_step("exit").expect("exit"),
            ScriptStep::ExitGame
        );
    }

    #[test]
    fn an_unknown_step_names_the_known_ones() {
        let error = parse_script_step("frob:1").expect_err("unknown");
        assert!(error.contains("frob"));
        assert!(error.contains("paint-from-to"));
    }

    #[test]
    fn a_missing_room_name_is_rejected() {
        assert!(parse_script_step("create:").is_err());
        assert!(parse_script_step("join:   ").is_err());
    }

    #[test]
    fn a_bad_count_is_rejected() {
        assert!(parse_script_step("wait-for-n-members:many").is_err());
        assert!(parse_script_step("wait-secs:soon").is_err());
    }

    #[test]
    fn a_bad_stroke_is_rejected() {
        assert!(parse_script_step("paint-from-to:10.20").is_err());
        assert!(parse_script_step("paint-from-to:10.20-9999.5").is_err());
    }

    /// The harness writes the script, this parser reads it. The same shape is
    /// pinned on the harness side (`tests/common/scenario.rs`).
    #[test]
    fn the_harness_script_shape_parses() {
        let script = "create:room-1, paint-from-to:200.300-260.300, \
                      paint-from-to:230.309-290.309, erase-from-to:200.300-260.300, \
                      wait-secs:45, leave:room-1";
        let steps: Vec<ScriptStep> = script
            .split(", ")
            .map(|step| parse_script_step(step).expect("step"))
            .collect();
        assert_eq!(steps.len(), 6);
        assert_eq!(
            steps.first(),
            Some(&ScriptStep::CreateRoom(identity::RoomName::from("room-1")))
        );
        assert_eq!(
            steps.last(),
            Some(&ScriptStep::LeaveRoom(identity::RoomName::from("room-1")))
        );
    }
}
