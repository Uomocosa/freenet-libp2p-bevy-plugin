use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamStatus {
    pub streamed_room: Option<String>,
    pub freshness: Option<String>,
    pub painted_tiles: usize,
    pub remote_cursors: usize,
    pub has_stream_image: bool,
}

impl StreamStatus {
    #[must_use]
    pub fn is_showing_room(&self, room: &str) -> bool {
        self.streamed_room.as_deref() == Some(room)
            && self.freshness.as_deref() == Some("Live")
            && self.painted_tiles > 0
            && self.has_stream_image
    }
}

#[cfg(test)]
mod tests {
    use super::StreamStatus;

    fn status() -> StreamStatus {
        StreamStatus {
            streamed_room: Some(String::from("alpha")),
            freshness: Some(String::from("Live")),
            painted_tiles: 3,
            remote_cursors: 1,
            has_stream_image: true,
        }
    }

    #[test]
    fn test_usage() {
        assert!(status().is_showing_room("alpha"));
        assert!(!status().is_showing_room("beta"));
        assert!(
            !StreamStatus {
                has_stream_image: false,
                ..status()
            }
            .is_showing_room("alpha")
        );
        assert!(
            !StreamStatus {
                painted_tiles: 0,
                ..status()
            }
            .is_showing_room("alpha")
        );
        assert!(
            !StreamStatus {
                freshness: Some(String::from("Bootstrapping")),
                ..status()
            }
            .is_showing_room("alpha")
        );
    }
}
