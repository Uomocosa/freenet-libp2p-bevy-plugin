#[must_use]
pub fn quarantine_corrupt_file(path: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut candidate = path.as_os_str().to_owned();
    candidate.push(".corrupt");
    let mut candidate = std::path::PathBuf::from(candidate);
    let mut suffix = 1_u32;
    while candidate.exists() {
        let mut numbered = path.as_os_str().to_owned();
        numbered.push(format!(".corrupt.{suffix}"));
        candidate = std::path::PathBuf::from(numbered);
        suffix = suffix.saturating_add(1);
    }
    std::fs::rename(path, &candidate).ok()?;
    Some(candidate)
}

#[cfg(test)]
mod tests {
    use super::quarantine_corrupt_file;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("book.json");
        std::fs::write(&path, b"not json").expect("write");
        let quarantined = quarantine_corrupt_file(path.as_path()).expect("quarantined");
        assert!(!path.exists());
        assert_eq!(std::fs::read(&quarantined).expect("read"), b"not json");
    }

    #[test]
    fn existing_quarantine_gets_a_number() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("book.json");
        std::fs::write(&path, b"second").expect("write");
        let mut taken = path.as_os_str().to_owned();
        taken.push(".corrupt");
        let taken = std::path::PathBuf::from(taken);
        std::fs::write(&taken, b"first").expect("write");
        let quarantined = quarantine_corrupt_file(path.as_path()).expect("quarantined");
        assert_ne!(quarantined, taken);
        assert_eq!(std::fs::read(&quarantined).expect("read"), b"second");
        assert_eq!(std::fs::read(&taken).expect("read"), b"first");
    }

    #[test]
    fn missing_file_quarantines_to_nothing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("missing.json");
        assert!(quarantine_corrupt_file(path.as_path()).is_none());
    }
}
