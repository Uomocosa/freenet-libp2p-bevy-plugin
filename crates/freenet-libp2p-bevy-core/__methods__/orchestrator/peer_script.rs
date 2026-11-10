use crate::testing;

pub fn peer_script(
    orchestrator: &testing::Orchestrator,
    peer_index: usize,
    room: &str,
) -> Result<String, String> {
    let Some(peer) = orchestrator.peers.get(peer_index) else {
        return Err("peer index out of range".to_string());
    };
    let mut parts: Vec<String> = Vec::new();
    for stored in &peer.steps {
        if stored.is_wait {
            let deadline = stored
                .within
                .unwrap_or_else(|| orchestrator.default_wait_deadline());
            parts.push(format!("{}@{}s", stored.text, deadline.as_secs()));
        } else if stored.within.is_some() {
            return Err(format!(
                "step '{}' is not a wait; only waits take within",
                stored.text
            ));
        } else if let Some(scaled) = orchestrator.scaled_sleep_text(&stored.text) {
            parts.push(scaled);
        } else {
            parts.push(stored.text.clone());
        }
    }
    Ok(parts.join(", ").replace(testing::ROOM_PLACEHOLDER, room))
}

#[cfg(test)]
mod tests {
    use super::peer_script;
    use crate::testing;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ToyWait;

    impl testing::traits::ScriptStep for ToyWait {
        const STEP_NAMES: &'static str = "toy-wait";

        fn parse_verb(verb: &str, _arg: &str) -> Option<Result<Self, String>> {
            match verb {
                "toy-wait" => Some(Ok(Self)),
                _ => None,
            }
        }

        fn is_wait(&self) -> bool {
            true
        }
    }

    impl testing::traits::OrchestratedStep for ToyWait {
        fn script_text(&self) -> String {
            "toy-wait".to_string()
        }
    }

    #[test]
    fn test_usage() {
        let mut orchestrator = testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::Fake,
        );
        orchestrator.peer("painter");
        let _ = testing::Orchestrator::runs(
            &mut orchestrator,
            [vec![testing::StepWithDeadline::from(ToyWait)]],
        );
        let script = peer_script(&orchestrator, 0, "room-1").expect("script");
        assert!(script.contains("toy-wait@"));
        assert!(peer_script(&orchestrator, 7, "room-1").is_err());
    }
}
