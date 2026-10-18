use crate::starboard;

#[must_use]
pub fn machine_info() -> starboard::bench::MachineInfo {
    starboard::bench::MachineInfo {
        kernel: command_output("uname", &["-srmo"])
            .unwrap_or_else(|| String::from(std::env::consts::OS)),
        cpu: cpu_model().unwrap_or_else(|| String::from(std::env::consts::ARCH)),
        cores: std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get),
    }
}

// needed helper: one command's trimmed stdout, or None when it cannot run
fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    Some(text.trim().to_string())
}

// needed helper: the processor model line of /proc/cpuinfo
fn cpu_model() -> Option<String> {
    let text = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    text.lines()
        .find(|line| line.starts_with("model name"))
        .and_then(|line| line.split_once(':'))
        .map(|(_, model)| model.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::machine_info;

    #[test]
    fn test_usage() {
        let info = machine_info();
        assert!(info.cores >= 1);
        assert!(!info.kernel.is_empty());
        assert!(!info.cpu.is_empty());
    }
}
