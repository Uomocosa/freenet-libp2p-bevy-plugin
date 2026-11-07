use atomic_delegate_macros::atomic_delegates;

use crate::testing;

pub struct Orchestrator {
    pub(crate) network: testing::Network,
    pub(crate) game: String,
    pub(crate) peers: Vec<testing::PeerBuilder>,
    pub should_check_canvas_convergence: bool,
    pub should_check_history_written: bool,
    pub min_members_override: Option<usize>,
    pub may_crash: std::collections::BTreeSet<String>,
    pub join_must_fail: std::collections::BTreeSet<String>,
}

impl Orchestrator {
    #[must_use]
    pub const fn from_game_and_network(game: String, network: testing::Network) -> Self {
        Self {
            game,
            network,
            peers: Vec::new(),
            should_check_canvas_convergence: false,
            should_check_history_written: false,
            min_members_override: None,
            may_crash: std::collections::BTreeSet::new(),
            join_must_fail: std::collections::BTreeSet::new(),
        }
    }

    pub fn peer(&mut self, name: &str) -> &mut Self {
        self.peers.push(testing::PeerBuilder::new(name.to_string()));
        self
    }

    pub fn with_token(&mut self, token: &str) -> &mut Self {
        if let Some(last) = self.peers.last_mut() {
            last.token_override = Some(token.to_string());
        }
        self
    }

    pub fn starts_after(&mut self, checkpoint: testing::Checkpoint) -> &mut Self {
        if let Some(last) = self.peers.last_mut() {
            last.starts_after = Some(checkpoint);
        }
        self
    }

    pub const fn expect_canvases_converge(&mut self) -> &mut Self {
        self.should_check_canvas_convergence = true;
        self
    }

    pub const fn expect_history_written_and_restored(&mut self) -> &mut Self {
        self.should_check_history_written = true;
        self
    }

    pub const fn expect_min_members(&mut self, count: usize) -> &mut Self {
        self.min_members_override = Some(count);
        self
    }

    pub fn expect_peer_may_crash(&mut self, name: &str) -> &mut Self {
        self.may_crash.insert(name.to_string());
        self
    }

    pub fn expect_join_must_fail(&mut self, name: &str) -> &mut Self {
        self.join_must_fail.insert(name.to_string());
        self
    }
}

#[rustfmt::skip]
impl Orchestrator {
    #[must_use]
    pub fn peer_names(&self) -> Vec<String> { self.peers.iter().map(|peer| peer.name.clone()).collect() }
}

#[atomic_delegates]
impl Orchestrator {
    pub fn runs<Step>(
        &mut self,
        steps: impl IntoIterator<Item = Vec<testing::StepWithDeadline<Step>>>,
    ) -> &mut Self
    where
        Step: testing::OrchestratedStep,
    {
    }

    pub async fn run(&mut self) -> testing::OrchestratorReport {}
}

#[rustfmt::skip]
impl Orchestrator {
    pub(crate) const fn default_wait_deadline(&self) -> std::time::Duration {
        match self.network {
            testing::Network::Fake => std::time::Duration::from_secs(90),
            testing::Network::Mainnet | testing::Network::LocalGateways => {
                std::time::Duration::from_secs(420)
            }
        }
    }

    pub(crate) fn time_scale(&self) -> f64 {
        let from_env = std::env::var("E2E_TIME_SCALE").ok().and_then(|raw| raw.parse::<f64>().ok()).filter(|scale| scale.is_finite() && *scale > 0.0);
        if let Some(scale) = from_env { return scale; }
        match self.network {
            testing::Network::Fake => 0.2,
            testing::Network::Mainnet | testing::Network::LocalGateways => 1.0,
        }
    }

    pub(crate) fn peer_token(&self, peer_index: usize, default_token: &str) -> String {
        self.peers.get(peer_index).and_then(|peer| peer.token_override.clone()).map_or_else(|| default_token.to_string(), |override_token| override_token.replace(testing::ROOM_PLACEHOLDER, default_token))
    }

    pub(crate) fn scaled_sleep_text(&self, text: &str) -> Option<String> {
        let secs = text.strip_prefix("sleep:")?.strip_suffix('s')?.parse::<u64>().ok()?;
        Some(format!("sleep:{}s", self.scaled_sleep_secs(secs)))
    }

    pub(crate) fn peer_identity_file(&self, token: &str, peer: &str) -> std::path::PathBuf {
        let safe_token = token.replace('/', "-");
        let file = format!(
            "{}-{safe_token}-pid{}-{peer}.identity",
            self.game,
            std::process::id()
        );
        std::env::temp_dir().join(file)
    }

    pub(crate) const fn cap(&self) -> std::time::Duration {
        match self.network {
            testing::Network::Fake => std::time::Duration::from_secs(300),
            testing::Network::Mainnet | testing::Network::LocalGateways => {
                std::time::Duration::from_mins(20)
            }
        }
    }

    pub(crate) fn network_arg(&self) -> String {
        match self.network {
            testing::Network::Fake => "fake".to_string(),
            testing::Network::Mainnet => "mainnet".to_string(),
            testing::Network::LocalGateways => "local-gateways".to_string(),
        }
    }

    pub(crate) fn early_report(
        game: &str,
        dir: std::path::PathBuf,
        detail: String,
    ) -> testing::OrchestratorReport {
        testing::OrchestratorReport {
            checks: vec![testing::ReportCheck {
                name: "setup".to_string(),
                ok: false,
                detail,
            }],
            logs_dir: dir,
            caption: format!("{game} setup failed"),
        }
    }

    pub(crate) fn poll_exits(running: &mut [testing::RunningPeer]) {
        for slot in running.iter_mut() {
            let Some(child) = slot.child.as_mut() else {
                continue;
            };
            if slot.exit.is_some() {
                continue;
            }
            if let Ok(Some(status)) = child.try_wait() {
                slot.exit = Some(
                    status
                        .code()
                        .map_or(testing::PeerExit::Signal, testing::PeerExit::Code),
                );
            }
        }
    }

    pub(crate) async fn spawn_due_peers(
        &self,
        running: &mut [testing::RunningPeer],
        dir: &std::path::Path,
        bin: &std::path::Path,
        scripts: &[String],
        token: &str,
        transport: &str,
        orchestrator_port: Option<u16>,
    ) {
        const SPAWN_GAP: std::time::Duration = std::time::Duration::from_millis(300);
        const DEFAULT_RUST_LOG: &str =
            "info,lobby_room=debug,starboard=debug,p2p=debug,discovery=debug,freenet_libp2p_bevy_plugin=debug";
        for index in 0..self.peers.len() {
            let already = running
                .get(index)
                .is_some_and(|peer| peer.child.is_some());
            if already {
                continue;
            }
            let starts_after = self
                .peers
                .get(index)
                .and_then(|peer| peer.starts_after);
            let due = starts_after.is_none_or(|checkpoint| {
                let needle = format!(
                    "{}{checkpoint}",
                    testing::SCRIPT_CHECKPOINT_COMPLETED_PREFIX
                );
                running.iter().any(|peer| {
                    std::fs::read_to_string(&peer.log)
                        .unwrap_or_default()
                        .contains(&needle)
                })
            });
            if !due {
                continue;
            }
            let Some(peer) = self.peers.get(index) else {
                continue;
            };
            let Some(script) = scripts.get(index) else {
                continue;
            };
            let log = dir.join(format!("instance-{}.log", peer.name));
            let Ok(log_file) = std::fs::File::create(&log) else {
                continue;
            };
            let Ok(err_file) = log_file.try_clone() else {
                continue;
            };
            let peer_token = self.peer_token(index, token);
            let mut args = vec![
                "--username".to_string(),
                peer.name.clone(),
                "--token".to_string(),
                peer_token,
                "--transport".to_string(),
                transport.to_string(),
                "--script".to_string(),
                script.clone(),
                "--network".to_string(),
                self.network_arg(),
            ];
            if let Some(port) = orchestrator_port {
                args.push("--orchestrator-port".to_string());
                args.push(port.to_string());
            }
            let node_dir = dir.join(format!("{}-freenet", peer.name));
            args.push("--freenet-node-dir".to_string());
            args.push(node_dir.display().to_string());
            let identity_file = self.peer_identity_file(token, &peer.name);
            args.push("--identity-file".to_string());
            args.push(identity_file.display().to_string());
            let result_path = dir.join(&peer.name).join("result.json");
            args.push("--result-json".to_string());
            args.push(result_path.display().to_string());
            let mut rust_log =
                std::env::var("LOBBY_RUST_LOG").unwrap_or_else(|_| DEFAULT_RUST_LOG.to_string());
            if !rust_log.contains("room_sync") {
                rust_log.push(',');
                rust_log.push_str("room_sync=info");
            }
            let scale = self.time_scale();
            let child = std::process::Command::new(bin)
                .args(&args)
                .env("RUST_LOG", rust_log)
                .env("E2E_TIME_SCALE", scale.to_string())
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::from(log_file))
                .stderr(std::process::Stdio::from(err_file))
                .spawn();
            if let Some(slot) = running.get_mut(index) {
                slot.child = child.ok();
            }
            tokio::time::sleep(SPAWN_GAP).await;
        }
    }
}

#[atomic_delegates]
impl Orchestrator {
    pub(crate) fn peer_script(&self, peer_index: usize, room: &str) -> Result<String, String> {}

    pub(crate) fn scaled_sleep_secs(&self, secs: u64) -> u64 {}

    pub(crate) fn build_game_binary(&self) -> Result<std::path::PathBuf, String> {}

    pub(crate) fn build_checks(
        &self,
        running: &[testing::RunningPeer],
    ) -> Vec<testing::ReportCheck> {
    }

    pub(crate) fn finalize_run(
        game: &str,
        room: &str,
        transport: &str,
        dir: std::path::PathBuf,
        running: &mut [testing::RunningPeer],
        checks: Vec<testing::ReportCheck>,
    ) -> testing::OrchestratorReport {
    }

    pub(crate) fn setup_run_dir(&self) -> Result<(std::path::PathBuf, String, String), String> {}

    pub(crate) async fn setup_bin_port_scripts(
        &self,
        room: &str,
    ) -> Result<(std::path::PathBuf, Option<u16>, Vec<String>), String> {
    }

    pub(crate) async fn drive_until_done(
        &mut self,
        running: &mut [testing::RunningPeer],
        dir: &std::path::Path,
        bin: &std::path::Path,
        scripts: &[String],
        token: &str,
        transport: &str,
        orchestrator_port: Option<u16>,
        cap: std::time::Duration,
        started: std::time::Instant,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::Orchestrator;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut orchestrator =
            Orchestrator::from_game_and_network("starboard".to_string(), testing::Network::Fake);
        orchestrator
            .peer("painter")
            .starts_after(testing::Checkpoint::new("painting-done"))
            .expect_canvases_converge();
        assert_eq!(orchestrator.peers.len(), 1);
        assert!(orchestrator.should_check_canvas_convergence);
    }

    #[test]
    fn peer_identity_files_are_distinct_and_outside_results() {
        use std::path::PathBuf;
        let orchestrator =
            Orchestrator::from_game_and_network("starboard".to_string(), testing::Network::Fake);
        let results = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".local-run");
        let alice = orchestrator.peer_identity_file("lobby-e2e/123", "alice");
        let bob = orchestrator.peer_identity_file("lobby-e2e/123", "bob");
        assert_ne!(alice, bob);
        assert_eq!(
            alice.extension().and_then(|ext| ext.to_str()),
            Some("identity")
        );
        assert!(!alice.starts_with(&results));
        assert!(!bob.starts_with(&results));
    }
}
