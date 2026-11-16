use std::path::Path;

use atomic_delegate_macros::atomic_delegates;
use derive_more::{Deref, From};

#[derive(Debug, Clone, Deref, From)]
pub struct ScreenCapture(pub(crate) String);

#[atomic_delegates]
impl ScreenCapture {
    pub fn ffmpeg_arguments(&self, clip: &Path) -> Vec<String> {}
    pub fn apply_environment(&self, command: &mut std::process::Command) {}
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::ScreenCapture;

    #[test]
    fn test_usage() {
        let capture = ScreenCapture::from(":91".to_string());
        assert_eq!(&*capture, ":91");
        let arguments = capture.ffmpeg_arguments(Path::new("/tmp/clip.mp4"));
        assert_eq!(arguments.last().map(String::as_str), Some("/tmp/clip.mp4"));
        let mut command = std::process::Command::new("ffmpeg");
        capture.apply_environment(&mut command);
    }
}
