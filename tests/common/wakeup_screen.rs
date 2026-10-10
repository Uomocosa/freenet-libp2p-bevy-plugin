use std::process::Command;

pub fn wakeup_screen() {
    let display = std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
    let _ = Command::new("cinnamon-screensaver-command")
        .arg("--deactivate")
        .output();
    let _ = Command::new("gnome-screensaver-command")
        .arg("--deactivate")
        .output();
    let _ = Command::new("xdg-screensaver").arg("reset").output();
    let _ = Command::new("xset")
        .args(["-display", &display, "dpms", "force", "on"])
        .output();
    let _ = Command::new("loginctl").arg("unlock-session").output();
    let _ = Command::new("xdotool")
        .args(["mousemove", "1", "1"])
        .output();
    let _ = Command::new("xdotool")
        .args(["mousemove", "2", "2"])
        .output();
    let _ = Command::new("xset").args(["s", "reset"]).output();
}
