use std::process::Command;

/// # Errors
/// Returns an error if no display is set, or if `xdotool`, `wmctrl` or `xdpyinfo` is missing.
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
