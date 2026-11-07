use crate::direct_dial;
use crate::testing;

#[must_use]
pub async fn check_book_kind(
    orchestrator: &testing::Orchestrator,
    token: &str,
    expected: &testing::BookKindExpectation,
) -> testing::ReportCheck {
    let wanted = match expected.kind {
        direct_dial::KnownPeerKind::Friend => "friend",
        direct_dial::KnownPeerKind::PreviouslyConnected => "previously-connected",
    };
    let name = format!(
        "{} book marks {} as {wanted}",
        expected.owner, expected.peer
    );
    let fail = |detail: String| testing::ReportCheck {
        name: name.clone(),
        ok: false,
        detail,
    };
    let id = if orchestrator.is_remote_peer(&expected.peer) {
        let Some(board) = orchestrator.board.clone() else {
            return fail(format!("no board to resolve {}", expected.peer));
        };
        match board.find_peer(&expected.peer).await {
            Ok(Some(found)) => found.id,
            _ => return fail(format!("no identity for {}", expected.peer)),
        }
    } else {
        let Ok(id) = orchestrator.peer_id_for_name(token, &expected.peer) else {
            return fail(format!("no identity for {}", expected.peer));
        };
        id
    };
    let path = orchestrator.peer_book_file(token, &expected.owner);
    let Ok(book) = direct_dial::PeerBook::try_from(path.as_path()) else {
        return fail(format!("book {} unreadable", path.display()));
    };
    let Some(entry) = book.iter().find(|entry| entry.peer.id == id) else {
        return fail(format!("{} has no entry", expected.peer));
    };
    testing::ReportCheck {
        name,
        ok: entry.kind == expected.kind,
        detail: format!("kind is {:?}", entry.kind),
    }
}

#[cfg(test)]
mod tests {
    use super::check_book_kind;
    use crate::direct_dial;
    use crate::identity;
    use crate::testing;

    fn expectation(kind: direct_dial::KnownPeerKind) -> testing::BookKindExpectation {
        testing::BookKindExpectation {
            owner: "alice".to_string(),
            peer: "bob".to_string(),
            kind,
        }
    }

    fn orchestrator() -> testing::Orchestrator {
        testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::DirectDial,
        )
    }

    #[tokio::test]
    async fn test_usage() {
        let id = orchestrator()
            .peer_id_for_name("kind-token", "bob")
            .expect("identity");
        let path = orchestrator().peer_book_file("kind-token", "alice");
        let mut book = direct_dial::PeerBook::default();
        book.push(direct_dial::KnownPeer {
            peer: identity::Peer {
                id,
                addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/5001")],
            },
            kind: direct_dial::KnownPeerKind::PreviouslyConnected,
        });
        book.save_to_file(path.as_path()).expect("book");
        let check = check_book_kind(
            &orchestrator(),
            "kind-token",
            &expectation(direct_dial::KnownPeerKind::PreviouslyConnected),
        )
        .await;
        assert!(check.ok, "unexpected failure: {}", check.detail);
        let check = check_book_kind(
            &orchestrator(),
            "kind-token",
            &expectation(direct_dial::KnownPeerKind::Friend),
        )
        .await;
        assert!(!check.ok);
        assert!(check.detail.contains("PreviouslyConnected"));
        std::fs::remove_file(&path).expect("cleanup");
        std::fs::remove_file(orchestrator().peer_identity_file("kind-token", "bob"))
            .expect("cleanup");
    }
}
