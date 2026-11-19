use crate::direct_dial;

#[must_use]
pub fn saved_peer_books(
    outputs: &[direct_dial::state_machine::Output],
) -> Vec<&direct_dial::PeerBook> {
    outputs
        .iter()
        .filter_map(|output| match output {
            direct_dial::state_machine::Output::SavePeerBook(book) => Some(book),
            direct_dial::state_machine::Output::Dial(_) => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::direct_dial;

    use super::saved_peer_books;

    #[test]
    fn test_usage() {
        let outputs = vec![direct_dial::state_machine::Output::SavePeerBook(
            direct_dial::PeerBook::default(),
        )];
        assert_eq!(saved_peer_books(&outputs).len(), 1);
    }
}
