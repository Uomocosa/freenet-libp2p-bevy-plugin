use crate::testing;

pub const fn script_step_is_wait(lobby_step: &testing::LobbyStep) -> bool {
    matches!(
        lobby_step,
        testing::LobbyStep::WaitLobbyRoom(_)
            | testing::LobbyStep::WaitLobbyRoomGone(_)
            | testing::LobbyStep::WaitJoinSucceeded(_)
            | testing::LobbyStep::WaitJoinFailed(_)
            | testing::LobbyStep::WaitLeft(_)
            | testing::LobbyStep::WaitRingJoined
            | testing::LobbyStep::WaitLobbyReady
    )
}

#[cfg(test)]
mod tests {
    use super::script_step_is_wait;
    use crate::testing;

    #[test]
    fn test_usage() {
        assert!(!script_step_is_wait(&testing::LobbyStep::ExitApp));
    }
}
