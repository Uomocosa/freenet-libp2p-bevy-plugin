use crate::p2p;

#[must_use]
pub fn render_history(kinds: &[p2p::ConnectionKind]) -> String {
    if kinds.is_empty() {
        return "none".to_string();
    }
    let mut names: Vec<&str> = Vec::new();
    for kind in kinds {
        let name = match kind {
            p2p::ConnectionKind::Tcp => "Tcp",
            p2p::ConnectionKind::Quic => "Quic",
            p2p::ConnectionKind::Relayed => "Relayed",
        };
        names.push(name);
    }
    let mut text = names.join(", ");
    if p2p::ConnectionKind::is_hole_punched(kinds) {
        text.push_str(" hole-punched");
    }
    text
}

#[cfg(test)]
mod tests {
    use super::render_history;
    use crate::p2p;

    #[test]
    fn test_usage() {
        assert_eq!(render_history(&[]), "none".to_string());
        assert_eq!(
            render_history(&[p2p::ConnectionKind::Tcp]),
            "Tcp".to_string()
        );
        assert!(
            render_history(&[p2p::ConnectionKind::Relayed, p2p::ConnectionKind::Quic])
                .contains("hole-punched")
        );
    }
}
