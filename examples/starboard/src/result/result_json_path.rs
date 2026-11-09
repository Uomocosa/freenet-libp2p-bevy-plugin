use std::path::PathBuf;

use bevy::prelude::Resource;
use derive_more::Deref;

#[derive(Resource, Deref)]
pub struct ResultJsonPath(pub PathBuf);

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::ResultJsonPath;

    #[test]
    fn test_usage() {
        let path = ResultJsonPath(PathBuf::from("/tmp/result.json"));
        assert_eq!(path.as_path(), PathBuf::from("/tmp/result.json").as_path());
    }
}
