use crate::parse_stroke::{parse_cell, parse_stroke};
use crate::shared::script::shared_script_step::SharedScriptStep;
use crate::shared::script::timed_script_step::ScriptStepGrammar;
use crate::starboard;

/// One step of a starboard `--script`: a shared step, or a canvas step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StarboardScriptStep {
    Shared(SharedScriptStep),
    PaintFromTo(starboard::canvas::Stroke),
    EraseFromTo(starboard::canvas::Stroke),
    WaitCanvasNonEmpty,
    WaitCanvasStable,
    WaitCellPainted(starboard::canvas::CellPos),
}

impl ScriptStepGrammar for StarboardScriptStep {
    const STEP_NAMES: &'static str = "create, join, leave, wait-lobby-room, \
        wait-join-succeeded, wait-join-failed, wait-left, wait-members, \
        wait-members-are-strictly-less-than, wait-members-are-less-or-equal-to, wait-room-messages, \
        sleep, exit-app, crash-app, stay, paint-from-to, erase-from-to, wait-canvas-nonempty, \
        wait-canvas-stable, wait-cell-painted";

    fn parse_verb(verb: &str, arg: &str) -> Option<Result<Self, String>> {
        match verb {
            "paint-from-to" => Some(parse_stroke(arg).map(Self::PaintFromTo)),
            "erase-from-to" => Some(parse_stroke(arg).map(Self::EraseFromTo)),
            "wait-canvas-nonempty" => Some(Ok(Self::WaitCanvasNonEmpty)),
            "wait-canvas-stable" => Some(Ok(Self::WaitCanvasStable)),
            "wait-cell-painted" => Some(parse_cell(arg).map(Self::WaitCellPainted)),
            _ => SharedScriptStep::parse_verb(verb, arg).map(|step| step.map(Self::Shared)),
        }
    }

    fn is_wait(&self) -> bool {
        match self {
            Self::Shared(step) => step.is_wait(),
            Self::WaitCanvasNonEmpty | Self::WaitCanvasStable | Self::WaitCellPainted(_) => true,
            Self::PaintFromTo(_) | Self::EraseFromTo(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use freenet_libp2p_bevy_plugin::identity;

    use super::StarboardScriptStep;
    use crate::shared::script::shared_script_step::SharedScriptStep;
    use crate::shared::script::timed_script_step::parse_timed_script_step;
    use crate::starboard;

    fn step(raw: &str) -> Result<StarboardScriptStep, String> {
        parse_timed_script_step::<StarboardScriptStep>(raw).map(|timed| timed.step)
    }

    #[test]
    fn test_usage() {
        assert_eq!(
            step("paint-from-to:10.20-30.102"),
            Ok(StarboardScriptStep::PaintFromTo(
                starboard::canvas::Stroke {
                    from: starboard::canvas::CellPos { x: 10, y: 20 },
                    to: starboard::canvas::CellPos { x: 30, y: 102 },
                }
            ))
        );
        assert_eq!(
            step("wait-canvas-stable@30s"),
            Ok(StarboardScriptStep::WaitCanvasStable)
        );
        assert_eq!(
            step("join:room-1"),
            Ok(StarboardScriptStep::Shared(SharedScriptStep::JoinRoom(
                identity::RoomName::from("room-1")
            )))
        );
    }

    #[test]
    fn canvas_waits_need_a_deadline() {
        assert!(step("wait-canvas-nonempty").is_err());
        assert!(step("wait-cell-painted:10.20").is_err());
    }

    #[test]
    fn a_cell_wait_parses_its_cell() {
        assert_eq!(
            step("wait-cell-painted:10.20@60s"),
            Ok(StarboardScriptStep::WaitCellPainted(
                starboard::canvas::CellPos { x: 10, y: 20 }
            ))
        );
        assert!(step("wait-cell-painted:9999.5@60s").is_err());
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
        let steps: Result<Vec<StarboardScriptStep>, String> =
            script.split(", ").map(step).collect();
        assert_eq!(steps.map(|steps| steps.len()), Ok(8));
    }

    #[test]
    fn bounded_member_waits_parse_in_starboard_scripts() {
        assert_eq!(
            step("wait-members-are-strictly-less-than:2@60s"),
            Ok(StarboardScriptStep::Shared(
                SharedScriptStep::WaitMembersAreStrictlyLessThan(
                    crate::shared::script::shared_script_step::MemberCount(2)
                )
            ))
        );
        assert_eq!(
            step("wait-members-are-less-or-equal-to:1@60s"),
            Ok(StarboardScriptStep::Shared(
                SharedScriptStep::WaitMembersAreLessOrEqualTo(
                    crate::shared::script::shared_script_step::MemberCount(1)
                )
            ))
        );
    }
}
