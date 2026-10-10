use std::process::Command;

/// # Errors
/// Returns an error if `xdotool`, `wmctrl` or `xdpyinfo` is missing.
pub fn require_x11() -> Result<(), String> {
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
