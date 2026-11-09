use derive_more::Deref;

use crate::identity;

#[derive(Debug, Clone, Deref)]
pub struct SavedKeypair(libp2p::identity::Keypair);

impl TryFrom<&std::path::Path> for SavedKeypair {
    type Error = identity::SavedKeypairError;

    fn try_from(path: &std::path::Path) -> Result<Self, Self::Error> {
        match std::fs::read(path) {
            Ok(bytes) => {
                let keypair = libp2p::identity::Keypair::from_protobuf_encoding(&bytes)?;
                Ok(Self(keypair))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let keypair = libp2p::identity::Keypair::generate_ed25519();
                let bytes = keypair.to_protobuf_encoding()?;
                if let Some(parent) = path.parent()
                    && !parent.as_os_str().is_empty()
                {
                    std::fs::create_dir_all(parent)?;
                }
                let mut options = std::fs::OpenOptions::new();
                options.write(true).create(true).truncate(true);
                #[cfg(unix)]
                {
                    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
                }
                let mut file = options.open(path)?;
                std::io::Write::write_all(&mut file, &bytes)?;
                std::io::Write::flush(&mut file)?;
                #[cfg(unix)]
                {
                    std::fs::set_permissions(
                        path,
                        std::os::unix::fs::PermissionsExt::from_mode(0o600),
                    )?;
                }
                Ok(Self(keypair))
            }
            Err(error) => Err(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::identity;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("id.identity");
        let first = identity::SavedKeypair::try_from(path.as_path()).expect("create");
        let second = identity::SavedKeypair::try_from(path.as_path()).expect("load");
        assert_eq!(first.public().to_peer_id(), second.public().to_peer_id());
    }

    #[test]
    fn distinct_paths_give_distinct_peers() {
        let dir = tempfile::tempdir().expect("tempdir");
        let first = identity::SavedKeypair::try_from(dir.path().join("a.identity").as_path())
            .expect("first");
        let second = identity::SavedKeypair::try_from(dir.path().join("b.identity").as_path())
            .expect("second");
        assert_ne!(first.public().to_peer_id(), second.public().to_peer_id());
    }

    #[test]
    fn corrupt_file_returns_error_without_overwriting() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("bad.identity");
        std::fs::write(&path, b"not a keypair").expect("write");
        let result = identity::SavedKeypair::try_from(path.as_path());
        assert!(result.is_err());
        let raw = std::fs::read(&path).expect("read");
        assert_eq!(raw, b"not a keypair");
    }

    #[cfg(unix)]
    #[test]
    fn created_file_has_owner_only_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("nested").join("id.identity");
        identity::SavedKeypair::try_from(path.as_path()).expect("create");
        let mode = std::fs::metadata(&path)
            .expect("metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }
}
