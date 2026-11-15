use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum NodeDir {
    Temporary(tempfile::TempDir),
    Kept(PathBuf),
}

impl TryFrom<Option<PathBuf>> for NodeDir {
    type Error = std::io::Error;

    fn try_from(kept_node_dir: Option<PathBuf>) -> Result<Self, Self::Error> {
        let Some(path) = kept_node_dir else {
            return tempfile::tempdir().map(Self::Temporary);
        };
        std::fs::create_dir_all(&path)?;
        Ok(Self::Kept(path))
    }
}

impl NodeDir {
    #[must_use]
    #[rustfmt::skip]
    pub fn path(&self) -> &Path { match self { Self::Temporary(temporary) => temporary.path(), Self::Kept(path) => path } }
}

#[cfg(test)]
mod tests {
    use crate::freenet_node;

    #[test]
    fn test_usage() {
        let temporary = freenet_node::NodeDir::try_from(None).expect("temporary");
        assert!(temporary.path().is_dir());
        let parent = tempfile::tempdir().expect("parent");
        let kept_path = parent.path().join("nested").join("node");
        let kept = freenet_node::NodeDir::try_from(Some(kept_path.clone())).expect("kept");
        assert!(kept_path.is_dir());
        assert_eq!(kept.path(), kept_path.as_path());
    }

    #[test]
    fn temporary_folder_is_removed_on_drop() {
        let temporary = freenet_node::NodeDir::try_from(None).expect("temporary");
        let path = temporary.path().to_path_buf();
        drop(temporary);
        assert!(!path.exists());
    }
}
