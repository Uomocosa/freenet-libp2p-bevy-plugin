use crate::testing;

pub fn explain(peer: &testing::harness::Peer, message: &str) -> String {
    let log = std::fs::read_to_string(&peer.log).unwrap_or_default();
    let lines: Vec<&str> = log
        .lines()
        .filter(|line| !line.contains(testing::harness::NOISE_MARKER))
        .collect();
    let tail = lines
        .iter()
        .skip(lines.len().saturating_sub(testing::harness::LOG_TAIL_LINES))
        .copied()
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "{message}\n--- log tail ({}) ---\n{tail}",
        peer.log.display()
    )
}

// no test_usage necessary
