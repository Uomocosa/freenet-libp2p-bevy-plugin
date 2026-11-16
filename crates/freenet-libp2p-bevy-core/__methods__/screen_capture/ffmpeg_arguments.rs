use std::path::Path;

use crate::testing;

pub fn ffmpeg_arguments(capture: &testing::harness::ScreenCapture, clip: &Path) -> Vec<String> {
    let mut arguments = ["-y", "-hide_banner", "-loglevel", "error"]
        .map(str::to_string)
        .to_vec();
    #[cfg(target_os = "linux")]
    arguments.extend(
        [
            "-f",
            "x11grab",
            "-framerate",
            testing::harness::E2E_RECORDING_FPS,
            "-video_size",
            testing::harness::E2E_CAPTURE_SIZE,
            "-draw_mouse",
            "0",
            "-i",
            capture.as_str(),
        ]
        .map(str::to_string),
    );
    arguments.extend(
        [
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-crf",
            testing::harness::E2E_RECORDING_CRF,
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
        ]
        .map(str::to_string),
    );
    arguments.push(clip.display().to_string());
    arguments
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::ffmpeg_arguments;
    use crate::testing;

    #[test]
    fn test_usage() {
        let capture = testing::harness::ScreenCapture::from(":91".to_string());
        let clip = Path::new("/tmp/run/e2e.mp4");
        let arguments = ffmpeg_arguments(&capture, clip);
        assert!(arguments.iter().any(|argument| argument == "-crf"));
        assert!(arguments.iter().any(|argument| argument == "libx264"));
        assert!(
            arguments
                .iter()
                .any(|argument| argument == testing::harness::E2E_RECORDING_CRF)
        );
        assert_eq!(arguments.last(), Some(&clip.display().to_string()));
        #[cfg(target_os = "linux")]
        {
            assert!(arguments.iter().any(|argument| argument == "x11grab"));
            assert!(arguments.iter().any(|argument| argument == ":91"));
            assert!(
                arguments
                    .iter()
                    .any(|argument| argument == testing::harness::E2E_CAPTURE_SIZE)
            );
            let draw_mouse = arguments
                .iter()
                .position(|argument| argument == "-draw_mouse")
                .expect("x11grab input hides the cursor");
            assert_eq!(arguments.get(draw_mouse + 1), Some(&"0".to_string()));
            let input = arguments
                .iter()
                .position(|argument| argument == "-i")
                .expect("x11grab input has a display");
            assert!(draw_mouse < input);
        }
    }
}
