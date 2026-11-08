#[cfg(not(any(target_os = "windows", target_os = "macos")))]
use std::process::Command;

/// # Errors
/// Returns an error if no display is set, or if `xdotool`, `wmctrl` or `xdpyinfo` is missing.
/// X is only used on Linux/BSD; on Windows and macOS there is no X display to require.
#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn require_x11() -> Result<(), String> {
    Ok(())
}

/// # Errors
/// Returns an error if no display is set, or if `xdotool`, `wmctrl` or `xdpyinfo` is missing.
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn require_x11() -> Result<(), String> {
    if std::env::var_os("DISPLAY").is_none() && std::env::var_os("WAYLAND_DISPLAY").is_none() {
        return Err(
            "no display: set DISPLAY (e.g. DISPLAY=:0 XAUTHORITY=~/.Xauthority) to run the visual e2e"
                .to_string(),
        );
    }
    for bin in ["xdotool", "wmctrl", "xdpyinfo"] {
        let ok = Command::new("which")
            .arg(bin)
            .output()
            .is_ok_and(|o| o.status.success());
        if !ok {
            return Err(format!("{bin} not found on PATH"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    #[test]
    fn test_usage() {
        assert!(super::require_x11().is_ok());
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    #[test]
    fn test_usage() {
        // SAFETY: nextest runs each test in its own process, so mutating the
        // environment here cannot race with another test.
        unsafe {
            std::env::remove_var("DISPLAY");
            std::env::remove_var("WAYLAND_DISPLAY");
        }
        assert!(super::require_x11().is_err());
    }
}
