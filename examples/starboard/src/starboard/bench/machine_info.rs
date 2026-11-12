#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MachineInfo {
    pub kernel: String,
    pub cpu: String,
    pub cores: usize,
}

impl MachineInfo {
    #[must_use]
    pub fn detect() -> Self {
        Self {
            kernel: Self::command_output("uname", &["-srmo"])
                .unwrap_or_else(|| String::from(std::env::consts::OS)),
            cpu: Self::cpu_model().unwrap_or_else(|| String::from(std::env::consts::ARCH)),
            cores: std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get),
        }
    }

    pub(crate) fn command_output(program: &str, args: &[&str]) -> Option<String> {
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

    pub(crate) fn cpu_model() -> Option<String> {
        let text = std::fs::read_to_string("/proc/cpuinfo").ok()?;
        text.lines()
            .find(|line| line.starts_with("model name"))
            .and_then(|line| line.split_once(':'))
            .map(|(_, model)| model.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::MachineInfo;

    #[test]
    fn test_usage() {
        let info = MachineInfo::detect();
        assert!(info.cores >= 1);
        assert_ne!(info.kernel, "");
        assert_ne!(info.cpu, "");
    }
}
