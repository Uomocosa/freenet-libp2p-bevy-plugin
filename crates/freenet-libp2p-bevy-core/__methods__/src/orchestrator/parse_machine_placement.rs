pub fn parse_machine_placement(raw: &str) -> Result<Vec<(String, String)>, String> {
    let mut pairs: Vec<(String, String)> = Vec::new();
    for fragment in raw.split(',') {
        let trimmed = fragment.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some((peer, machine)) = trimmed.split_once('=') else {
            return Err(format!("placement '{trimmed}' needs peer=machine"));
        };
        let peer = peer.trim();
        let machine = machine.trim();
        if peer.is_empty() || machine.is_empty() {
            return Err(format!("placement '{trimmed}' needs peer=machine"));
        }
        if pairs.iter().any(|(placed, _)| placed == peer) {
            return Err(format!("peer '{peer}' is placed twice"));
        }
        pairs.push((peer.to_string(), machine.to_string()));
    }
    Ok(pairs)
}

#[cfg(test)]
mod tests {
    use super::parse_machine_placement;

    #[test]
    fn test_usage() {
        assert_eq!(
            parse_machine_placement("alice=a,bob=b").expect("placed"),
            vec![
                ("alice".to_string(), "a".to_string()),
                ("bob".to_string(), "b".to_string())
            ]
        );
        assert_eq!(
            parse_machine_placement("  alice = a , bob = b ").expect("placed"),
            vec![
                ("alice".to_string(), "a".to_string()),
                ("bob".to_string(), "b".to_string())
            ]
        );
        assert_eq!(parse_machine_placement("").expect("empty"), vec![]);
        assert_eq!(parse_machine_placement("  ").expect("blank"), vec![]);
    }

    #[test]
    fn malformed_fragments_are_rejected() {
        assert!(parse_machine_placement("alice").is_err());
        assert!(parse_machine_placement("=a").is_err());
        assert!(parse_machine_placement("alice=").is_err());
        assert!(parse_machine_placement("alice=a,bob").is_err());
        assert!(parse_machine_placement("alice=a,alice=b").is_err());
    }
}
