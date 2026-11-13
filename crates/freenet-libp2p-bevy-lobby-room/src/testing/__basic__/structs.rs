use crate::discovery;

pub struct LobbyStepView<'a> {
    pub status: &'a discovery::state_machine::DiscoveryState,
    pub join_error: Option<&'a str>,
}

// no test_usage necessary
