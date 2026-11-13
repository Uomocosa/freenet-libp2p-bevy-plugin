use crate::discovery;

impl From<&discovery::state_machine::Output> for Option<discovery::Event> {
    fn from(output: &discovery::state_machine::Output) -> Self {
        match output {
            discovery::state_machine::Output::Notify(event) => Some(event.clone()),
            discovery::state_machine::Output::Libp2pCommand(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::discovery::state_machine::Output;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let notify = Output::Notify(discovery::Event::LobbyUpdated);
        assert_eq!(
            Option::<discovery::Event>::from(&notify),
            Some(discovery::Event::LobbyUpdated)
        );
        let command = Output::Libp2pCommand(p2p::Libp2pCommand::MeasureHistoryStore);
        assert_eq!(Option::<discovery::Event>::from(&command), None);
    }
}
