use bevy::prelude::*;

use crate::testing;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptVerdict {
    Running,
    Finished,
    Failed,
}

#[rustfmt::skip]
impl ScriptVerdict {
    #[must_use]
    pub const fn app_exit_for_verdict(self) -> Option<AppExit> {
        match self {
            Self::Running => None,
            Self::Finished => Some(AppExit::Success),
            Self::Failed => Some(AppExit::from_code(testing::SCRIPT_FAILED_EXIT_CODE)),
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::ScriptVerdict;
    use crate::testing;

    #[test]
    fn test_usage() {
        assert_eq!(ScriptVerdict::Running.app_exit_for_verdict(), None);
        assert_eq!(
            ScriptVerdict::Finished.app_exit_for_verdict(),
            Some(AppExit::Success)
        );
        assert_eq!(
            ScriptVerdict::Failed.app_exit_for_verdict(),
            Some(AppExit::from_code(testing::SCRIPT_FAILED_EXIT_CODE))
        );
    }
}
