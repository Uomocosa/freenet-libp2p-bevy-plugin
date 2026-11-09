use crate::direct_dial;

pub fn save_to_file(
    book: &direct_dial::PeerBook,
    path: &std::path::Path,
) -> Result<(), direct_dial::PeerBookError> {
    let bytes = serde_json::to_vec_pretty(&**book)?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, &bytes)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::save_to_file;
    use crate::direct_dial;
    use crate::identity;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("book.json");
        let mut book = direct_dial::PeerBook::default();
        assert!(book.add_friend(identity::Peer {
            id: identity::PeerId::from("alice"),
            addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
        }));
        save_to_file(&book, path.as_path()).expect("save");
        assert!(path.exists());
        assert!(!path.with_extension("tmp").exists());
        let raw = std::fs::read(&path).expect("read");
        let decoded: direct_dial::PeerBook = serde_json::from_slice(&raw).expect("decode");
        assert_eq!(decoded, book);
    }

    #[test]
    fn save_is_atomic_leaving_no_temp_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("book.json");
        let book = direct_dial::PeerBook::default();
        save_to_file(&book, path.as_path()).expect("first save");
        save_to_file(&book, path.as_path()).expect("second save");
        assert!(!path.with_extension("tmp").exists());
        let loaded = direct_dial::PeerBook::try_from(path.as_path()).expect("load");
        assert_eq!(loaded, book);
    }
}
