use crate::testing;

pub const fn script_step_is_wait(e2e_step: &testing::E2EStep) -> bool {
    matches!(
        e2e_step,
        testing::E2EStep::WaitMembers(_)
            | testing::E2EStep::WaitMembersAreStrictlyLessThan(_)
            | testing::E2EStep::WaitMembersAreLessOrEqualTo(_)
            | testing::E2EStep::WaitRoomMessages(_)
    )
}

#[cfg(test)]
mod tests {
    use super::script_step_is_wait;
    use crate::testing;

    #[test]
    fn test_usage() {
        assert!(!script_step_is_wait(&testing::E2EStep::Stay));
    }
}
