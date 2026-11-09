use crate::identity;

#[must_use]
pub fn parse_ready_line(line: &str) -> Option<(identity::PeerId, Vec<identity::PeerAddr>)> {
    let (_, after) = line.split_once("ready peer_id=")?;
    let (id, rest) = after.split_once(' ')?;
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    let mut addrs = Vec::new();
    for (index, part) in rest.split('"').enumerate() {
        if index % 2 == 1 && !part.is_empty() {
            let mapped = part.replace("/ip4/0.0.0.0/", "/ip4/127.0.0.1/");
            addrs.push(identity::PeerAddr::from(mapped));
        }
    }
    if addrs.is_empty() {
        return None;
    }
    addrs.sort_by_key(|addr| !addr.contains("/tcp/"));
    Some((identity::PeerId::from(id), addrs))
}

#[cfg(test)]
mod tests {
    use super::parse_ready_line;
    use crate::identity;

    fn line() -> String {
        format!(
            "2026-01-01T00:00:00Z  INFO p2p: ready peer_id={} addrs=[PeerAddr(\"/ip4/0.0.0.0/udp/5002/quic-v1\"), PeerAddr(\"/ip4/0.0.0.0/tcp/5001\")]",
            "12D3KooWTest"
        )
    }

    #[test]
    fn test_usage() {
        let (id, addrs) = parse_ready_line(&line()).expect("ready line");
        assert_eq!(id, identity::PeerId::from("12D3KooWTest"));
        assert_eq!(
            addrs,
            vec![
                identity::PeerAddr::from("/ip4/127.0.0.1/tcp/5001"),
                identity::PeerAddr::from("/ip4/127.0.0.1/udp/5002/quic-v1"),
            ]
        );
    }

    #[test]
    fn noise_lines_have_no_addresses() {
        assert_eq!(parse_ready_line("script step done step=1/2 stay"), None);
        assert_eq!(parse_ready_line("ready peer_id="), None);
        assert_eq!(parse_ready_line("ready peer_id=abc addrs=[]"), None);
    }
}
