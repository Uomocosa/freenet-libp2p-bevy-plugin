use derive_more::{Deref, From};
use freenet_libp2p_bevy_plugin::testing::ScriptStep;

use e2e_steps::starboard_steps::StarboardStep;

#[derive(Debug, Clone, PartialEq, Eq, Deref, From)]
pub struct ScriptedStarboardStep(pub StarboardStep);

impl ScriptStep for ScriptedStarboardStep {
    const STEP_NAMES: &'static str = StarboardStep::STEP_NAMES;

    fn parse_verb(verb: &str, arg: &str) -> Option<Result<Self, String>> {
        StarboardStep::parse_verb(verb, arg).map(|result| result.map(Self::from))
    }

    fn is_wait(&self) -> bool {
        self.0.is_wait()
    }
}
