use crate::discovery;

pub const fn take_subscribe_answer(state: &mut discovery::LobbyContractSubscription) -> bool {
    let has_unread = state.has_unread_subscribe_answer;
    state.has_unread_subscribe_answer = false;
    has_unread
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use freenet_stdlib::prelude::{ContractCode, Parameters, WrappedContract};

    use super::take_subscribe_answer;
    use crate::discovery;
    use crate::freenet;

    #[test]
    fn test_usage() {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let mut subscription = discovery::LobbyContractSubscription {
            client: freenet::Client { write, read },
            key: WrappedContract::new(
                Arc::new(ContractCode::from(vec![0; 4])),
                Parameters::from(Vec::new()),
            )
            .key,
            lobby: discovery::Lobby::default(),
            has_unread_subscribe_answer: true,
        };
        assert!(take_subscribe_answer(&mut subscription));
        assert!(!take_subscribe_answer(&mut subscription));
    }
}
