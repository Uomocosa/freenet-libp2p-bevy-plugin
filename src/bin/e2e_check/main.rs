use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use freenet_libp2p_bevy_plugin::e2e_report;

#[derive(Debug, Parser)]
#[command(name = "e2e-check")]
struct Args {
    #[arg(long)]
    results_dir: PathBuf,
    #[arg(long)]
    peer: Vec<String>,
    #[arg(long)]
    expect: Vec<String>,
}

impl Args {
    pub(crate) fn expectation(&self) -> Result<e2e_report::ScenarioExpectation, String> {
        let mut canvas_converges = false;
        let mut min_members = 0;
        let mut may_crash = BTreeSet::new();
        for raw in &self.expect {
            if raw == "converge" {
                canvas_converges = true;
            } else if let Some(count) = raw.strip_prefix("min-members:") {
                let parsed = count
                    .parse::<usize>()
                    .map_err(|_| format!("bad --expect '{raw}'; want min-members:N"))?;
                min_members = parsed;
            } else if let Some(user) = raw.strip_prefix("may-crash:") {
                if user.is_empty() {
                    return Err(format!("bad --expect '{raw}'; want may-crash:<user>"));
                }
                may_crash.insert(user.to_string());
            } else {
                return Err(format!(
                    "bad --expect '{raw}'; want converge, min-members:N or may-crash:<user>"
                ));
            }
        }
        Ok(e2e_report::ScenarioExpectation {
            canvas_converges,
            min_members,
            may_crash,
        })
    }

    pub(crate) fn run(&self) -> Result<bool, String> {
        let expectation = self.expectation()?;
        let mut names: BTreeSet<String> = self.peer.iter().cloned().collect();
        let entries = std::fs::read_dir(&self.results_dir)
            .map_err(|error| format!("results-dir: {error}"))?;
        for entry in entries {
            let entry = entry.map_err(|error| format!("results-dir: {error}"))?;
            if entry
                .file_type()
                .map_err(|error| format!("results-dir: {error}"))?
                .is_dir()
                && let Some(name) = entry.file_name().to_str()
            {
                names.insert(name.to_string());
            }
        }
        let mut results: BTreeMap<String, Option<e2e_report::PeerRunResult>> = BTreeMap::new();
        for name in names {
            let path = self.results_dir.join(&name).join("result.json");
            match std::fs::read(&path) {
                Ok(bytes) => {
                    let parsed: e2e_report::PeerRunResult = serde_json::from_slice(&bytes)
                        .map_err(|error| format!("{}: {error}", path.display()))?;
                    results.insert(name, Some(parsed));
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    results.insert(name, None);
                }
                Err(error) => return Err(format!("{}: {error}", path.display())),
            }
        }
        let rows = expectation.check_peer_run_results(&results);
        print!(
            "{}",
            e2e_report::ExpectationResult::render_markdown_tables(&rows)
        );
        Ok(rows.iter().all(|row| row.pass))
    }
}

fn main() -> ExitCode {
    let args = Args::parse();
    match args.run() {
        Ok(pass) => {
            if pass {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(error) => {
            eprintln!("e2e-check: {error}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Args;
    use clap::Parser;

    #[test]
    fn test_usage() {
        let args = Args::try_parse_from([
            "e2e-check",
            "--results-dir",
            "/tmp/results",
            "--peer",
            "p1",
            "--expect",
            "converge",
            "--expect",
            "min-members:1",
            "--expect",
            "may-crash:p3",
        ])
        .expect("args");
        let expectation = args.expectation().expect("expectation");
        assert!(expectation.canvas_converges);
        assert_eq!(expectation.min_members, 1);
        assert!(expectation.may_crash.contains("p3"));
    }

    #[test]
    fn a_bad_expect_is_an_error() {
        let args =
            Args::try_parse_from(["e2e-check", "--results-dir", "/tmp", "--expect", "bogus"])
                .expect("args");
        assert!(args.expectation().is_err());
    }
}
