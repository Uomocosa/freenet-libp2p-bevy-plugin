use crate::discovery;
use crate::p2p;

impl From<&discovery::state_machine::Output> for Option<p2p::Libp2pCommand> {
    fn from(output: &discovery::state_machine::Output) -> Self {
        match output {
            discovery::state_machine::Output::Libp2pCommand(command) => Some(command.clone()),
            discovery::state_machine::Output::Notify(_) => None,
        }
    }
}

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

    #[test]
    fn libp2p_command_outputs_forward_their_command() {
        let command = Output::Libp2pCommand(p2p::Libp2pCommand::MeasureHistoryStore);
        assert_eq!(
            Option::<p2p::Libp2pCommand>::from(&command),
            Some(p2p::Libp2pCommand::MeasureHistoryStore)
        );
        let notify = Output::Notify(discovery::Event::LobbyUpdated);
        assert_eq!(Option::<p2p::Libp2pCommand>::from(&notify), None);
    }
}
