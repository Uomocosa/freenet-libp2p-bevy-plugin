use atomic_delegate_macros::atomic_delegates;

use crate::direct_dial;
use crate::identity;
use crate::testing;

pub struct Orchestrator {
    pub(crate) network: testing::Network,
    pub(crate) game: String,
    pub(crate) peers: Vec<testing::PeerBuilder>,
    pub(crate) board: Option<std::sync::Arc<dyn testing::CheckpointBoard>>,
    pub(crate) published_board_peers: std::collections::HashMap<String, String>,
    pub(crate) marked_board_checkpoints: std::collections::HashSet<String>,
    pub(crate) machine: String,
    pub(crate) placement: Vec<(String, String)>,
    pub should_check_canvas_convergence: bool,
    pub should_check_history_written: bool,
    pub should_forbid_freenet_node: bool,
    pub book_kind_expectations: Vec<testing::BookKindExpectation>,
    pub min_members_override: Option<usize>,
    pub may_crash: std::collections::BTreeSet<String>,
    pub join_must_fail: std::collections::BTreeSet<String>,
}

impl Orchestrator {
    #[must_use]
    pub fn from_game_and_network(game: String, network: testing::Network) -> Self {
        Self {
            game,
            network,
            peers: Vec::new(),
            board: None,
            published_board_peers: std::collections::HashMap::new(),
            marked_board_checkpoints: std::collections::HashSet::new(),
            machine: String::new(),
            placement: Vec::new(),
            should_check_canvas_convergence: false,
            should_check_history_written: false,
            should_forbid_freenet_node: false,
            book_kind_expectations: Vec::new(),
            min_members_override: None,
            may_crash: std::collections::BTreeSet::new(),
            join_must_fail: std::collections::BTreeSet::new(),
        }
    }

    pub fn set_checkpoint_board(
        &mut self,
        board: std::sync::Arc<dyn testing::CheckpointBoard>,
    ) -> &mut Self {
        self.board = Some(board);
        self
    }

    pub fn peer(&mut self, name: &str) -> &mut Self {
        self.peers.push(testing::PeerBuilder::new(name.to_string()));
        self
    }

    pub fn relaunch(&mut self, name: &str) -> &mut Self {
        self.peers.push(testing::PeerBuilder::new(name.to_string()));
        if let Some(last) = self.peers.last_mut() {
            last.starts_after_exit_of = Some(name.to_string());
        }
        self
    }

    pub fn with_token(&mut self, token: &str) -> &mut Self {
        if let Some(last) = self.peers.last_mut() {
            last.token_override = Some(token.to_string());
        }
        self
    }

    pub fn starts_after(&mut self, checkpoint: testing::ScriptCheckpoint) -> &mut Self {
        if let Some(last) = self.peers.last_mut() {
            last.starts_after = Some(checkpoint);
        }
        self
    }

    pub const fn expect_no_freenet_node(&mut self) -> &mut Self {
        self.should_forbid_freenet_node = true;
        self
    }

    pub fn expect_book_kind(
        &mut self,
        owner: &str,
        peer: &str,
        kind: direct_dial::KnownPeerKind,
    ) -> &mut Self {
        self.book_kind_expectations
            .push(testing::BookKindExpectation {
                owner: owner.to_string(),
                peer: peer.to_string(),
                kind,
            });
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

    #[must_use]
    pub fn local_peer_names(&self) -> Vec<String> {
        self.peers
            .iter()
            .filter(|peer| Self::peer_runs_here(&peer.name, &self.machine, &self.placement))
            .map(|peer| peer.name.clone())
            .collect()
    }
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

    pub(crate) fn peer_id_for_name(
        &self,
        token: &str,
        name: &str,
    ) -> Result<identity::PeerId, String> {
    }

    pub(crate) fn live_peer_addresses(
        &self,
        running: &[testing::RunningPeer],
    ) -> Vec<testing::LivePeer> {
    }

    pub(crate) fn parse_ready_line(
        line: &str,
    ) -> Option<(identity::PeerId, Vec<identity::PeerAddr>)> {
    }

    pub(crate) fn substituted_script(
        &self,
        peer_index: usize,
        room: &str,
        token: &str,
        live: &[testing::LivePeer],
    ) -> Option<String> {
    }

    pub(crate) fn substitute_fragment(
        &self,
        fragment: &str,
        token: &str,
        live: &[testing::LivePeer],
    ) -> Option<String> {
    }

    pub(crate) fn wait_peer_id(
        &self,
        name: &str,
        token: &str,
        live: &[testing::LivePeer],
    ) -> Option<identity::PeerId> {
    }

    pub(crate) async fn check_book_kind(
        &self,
        token: &str,
        expected: &testing::BookKindExpectation,
    ) -> testing::ReportCheck {
    }

    pub(crate) fn runs_here(&self, index: usize) -> bool {}

    pub(crate) fn is_remote_peer(&self, name: &str) -> bool {}

    pub(crate) fn discover_lan_ip() -> Option<String> {}

    pub(crate) fn routable_board_peer(peer: &testing::LivePeer) -> identity::Peer {}

    pub(crate) fn parse_machine_placement(raw: &str) -> Result<Vec<(String, String)>, String> {}

    pub(crate) async fn publish_ready_peers(
        &mut self,
        running: &[testing::RunningPeer],
        token: &str,
    ) {
    }

    pub(crate) async fn remote_board_peers(
        &self,
        running: &[testing::RunningPeer],
    ) -> Vec<testing::LivePeer> {
    }

    pub(crate) async fn run_check_role(
        &self,
        dir: &std::path::Path,
    ) -> testing::OrchestratorReport {
    }

    pub(crate) fn check_peer_dir(&self, dir: &std::path::Path, name: &str) -> testing::ReportCheck {
    }

    pub(crate) fn check_results_dir() -> Result<std::path::PathBuf, String> {}
}

impl Orchestrator {
    pub(crate) fn peer_runs_here(
        peer: &str,
        machine: &str,
        placement: &[(String, String)],
    ) -> bool {
        if placement.is_empty() {
            return true;
        }
        placement
            .iter()
            .any(|(placed_peer, placed_machine)| placed_peer == peer && placed_machine == machine)
    }
}

#[rustfmt::skip]
impl Orchestrator {
    pub(crate) fn this_machine() -> String { std::env::var("E2E_MACHINE").map_or_else(|_| String::new(), |raw| raw.trim().to_string()) }
    pub(crate) fn is_check_role() -> bool { std::env::var("E2E_ROLE").is_ok_and(|raw| raw.trim() == "check") }
}

#[rustfmt::skip]
impl Orchestrator {
    pub(crate) const fn default_wait_deadline(&self) -> std::time::Duration {
        match self.network {
            testing::Network::Fake => std::time::Duration::from_secs(90),
            testing::Network::Mainnet | testing::Network::LocalGateways | testing::Network::DirectDial => {
                std::time::Duration::from_secs(420)
            }
        }
    }

    pub(crate) fn time_scale(&self) -> f64 {
        let from_env = std::env::var("E2E_TIME_SCALE").ok().and_then(|raw| raw.parse::<f64>().ok()).filter(|scale| scale.is_finite() && *scale > 0.0);
        if let Some(scale) = from_env { return scale; }
        match self.network {
            testing::Network::Fake => 0.2,
            testing::Network::Mainnet | testing::Network::LocalGateways | testing::Network::DirectDial => 1.0,
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

    pub(crate) fn peer_book_file(&self, token: &str, peer: &str) -> std::path::PathBuf {
        let safe_token = token.replace('/', "-");
        let file = format!(
            "{}-{safe_token}-pid{}-{peer}.friends.json",
            self.game,
            std::process::id()
        );
        std::env::temp_dir().join(file)
    }

    pub(crate) const fn cap(&self) -> std::time::Duration {
        match self.network {
            testing::Network::Fake => std::time::Duration::from_secs(300),
            testing::Network::DirectDial => std::time::Duration::from_mins(10),
            testing::Network::Mainnet | testing::Network::LocalGateways => {
                std::time::Duration::from_mins(20)
            }
        }
    }

    pub(crate) fn network_arg(&self) -> String {
        match self.network {
            testing::Network::Fake => "fake".to_string(),
            testing::Network::DirectDial => "direct-dial".to_string(),
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
}

#[atomic_delegates]
impl Orchestrator {
    pub(crate) fn peer_script(&self, peer_index: usize, room: &str) -> Result<String, String> {}

    pub(crate) async fn spawn_due_peers(
        &self,
        running: &mut [testing::RunningPeer],
        dir: &std::path::Path,
        bin: &std::path::Path,
        room: &str,
        scripts: &[String],
        token: &str,
        transport: &str,
        orchestrator_port: Option<u16>,
    ) {
    }

    pub(crate) async fn prepare_peer_spawn(
        &self,
        index: usize,
        room: &str,
        scripts: &[String],
        token: &str,
        running: &[testing::RunningPeer],
        live: &[testing::LivePeer],
        dir: &std::path::Path,
    ) -> Option<(String, std::path::PathBuf)> {
    }

    pub(crate) fn scaled_sleep_secs(&self, secs: u64) -> u64 {}

    pub(crate) fn build_game_binary(&self) -> Result<std::path::PathBuf, String> {}

    pub(crate) async fn build_checks(
        &self,
        running: &[testing::RunningPeer],
        token: &str,
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
        room: &str,
        scripts: &[String],
        token: &str,
        transport: &str,
        orchestrator_port: Option<u16>,
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
            .starts_after(testing::ScriptCheckpoint::new("painting-done"))
            .expect_canvases_converge();
        assert_eq!(orchestrator.peers.len(), 1);
        assert!(orchestrator.should_check_canvas_convergence);
    }

    #[test]
    fn relaunch_reuses_the_peer_name_with_an_exit_gate() {
        let mut orchestrator =
            Orchestrator::from_game_and_network("starboard".to_string(), testing::Network::Fake);
        orchestrator.peer("bob");
        orchestrator.relaunch("bob");
        assert_eq!(orchestrator.peers.len(), 2);
        assert_eq!(orchestrator.peers[1].name, "bob");
        assert_eq!(
            orchestrator.peers[1].starts_after_exit_of,
            Some("bob".to_string())
        );
        assert!(orchestrator.peers[0].starts_after_exit_of.is_none());
    }

    #[test]
    fn local_peer_names_follow_placement() {
        let mut orchestrator =
            Orchestrator::from_game_and_network("starboard".to_string(), testing::Network::Fake);
        orchestrator.peer("alice");
        orchestrator.peer("bob");
        assert_eq!(
            orchestrator.local_peer_names(),
            vec!["alice".to_string(), "bob".to_string()]
        );
        orchestrator.machine = "a".to_string();
        orchestrator.placement = vec![
            ("alice".to_string(), "a".to_string()),
            ("bob".to_string(), "b".to_string()),
        ];
        assert_eq!(orchestrator.local_peer_names(), vec!["alice".to_string()]);
    }

    #[test]
    fn peers_run_here_without_a_split() {
        assert!(Orchestrator::peer_runs_here("alice", "", &[]));
        assert!(Orchestrator::peer_runs_here("alice", "a", &[]));
    }

    #[test]
    fn peers_run_only_on_their_placed_machine() {
        let placement = vec![
            ("alice".to_string(), "a".to_string()),
            ("bob".to_string(), "b".to_string()),
        ];
        assert!(Orchestrator::peer_runs_here("alice", "a", &placement));
        assert!(!Orchestrator::peer_runs_here("alice", "b", &placement));
        assert!(!Orchestrator::peer_runs_here("bob", "a", &placement));
        assert!(!Orchestrator::peer_runs_here("mallory", "a", &placement));
    }

    #[test]
    fn expectations_land_on_the_last_peer() {
        use crate::direct_dial;

        let mut orchestrator =
            Orchestrator::from_game_and_network("starboard".to_string(), testing::Network::Fake);
        orchestrator
            .peer("alice")
            .expect_no_freenet_node()
            .expect_book_kind(
                "alice",
                "bob",
                direct_dial::KnownPeerKind::PreviouslyConnected,
            );
        assert!(orchestrator.should_forbid_freenet_node);
        assert_eq!(orchestrator.book_kind_expectations.len(), 1);
    }

    #[test]
    fn peer_book_files_are_distinct_and_outside_results() {
        use std::path::PathBuf;
        let orchestrator =
            Orchestrator::from_game_and_network("starboard".to_string(), testing::Network::Fake);
        let results = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".local-run");
        let alice = orchestrator.peer_book_file("lobby-e2e/123", "alice");
        let bob = orchestrator.peer_book_file("lobby-e2e/123", "bob");
        assert_ne!(alice, bob);
        assert_eq!(alice.extension().and_then(|ext| ext.to_str()), Some("json"));
        assert!(!alice.starts_with(&results));
        assert!(!bob.starts_with(&results));
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
