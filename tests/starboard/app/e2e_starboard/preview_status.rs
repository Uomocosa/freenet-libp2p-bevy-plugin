use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewStatus {
    pub selected_room: Option<String>,
    pub freshness: Option<String>,
    pub painted_tiles: usize,
    pub remote_cursors: usize,
    pub has_preview_image: bool,
}

impl PreviewStatus {
    #[must_use]
    pub fn is_showing_room(&self, room: &str) -> bool {
        self.selected_room.as_deref() == Some(room)
            && self.freshness.as_deref() == Some("Live")
            && self.painted_tiles > 0
            && self.has_preview_image
    }
}

#[cfg(test)]
mod tests {
    use super::PreviewStatus;

    fn status() -> PreviewStatus {
        PreviewStatus {
            selected_room: Some(String::from("alpha")),
            freshness: Some(String::from("Live")),
            painted_tiles: 3,
            remote_cursors: 1,
            has_preview_image: true,
        }
    }

    #[test]
    fn test_usage() {
        assert!(status().is_showing_room("alpha"));
        assert!(!status().is_showing_room("beta"));
        assert!(
            !PreviewStatus {
                has_preview_image: false,
                ..status()
            }
            .is_showing_room("alpha")
        );
        assert!(
            !PreviewStatus {
                painted_tiles: 0,
                ..status()
            }
            .is_showing_room("alpha")
        );
        assert!(
            !PreviewStatus {
                freshness: Some(String::from("Bootstrapping")),
                ..status()
            }
            .is_showing_room("alpha")
        );
    }
}
