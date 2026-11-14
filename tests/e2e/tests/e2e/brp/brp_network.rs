use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use examples_shared::freenet_node_dir::FreenetNodeDir;
use freenet_libp2p_bevy_plugin::lobby_room::testing::LobbyKit;
use freenet_libp2p_bevy_plugin::testing::{FakeNetworkServer, Network};

use super::Deadlines;
use crate::common::{LobbySetup, LobbyToken, check_or_deploy};

/// The network one BRP run uses, set up the way the old harness set it up for the same network.
///
/// Fake: an in-process lobby server; peers get `--orchestrator-port`, and their token is unique.
/// Mainnet: `check_or_deploy` makes the shared `lobby-e2e/ensured` contract exist first, and each
/// peer gets its own `--freenet-node-dir`. `LocalGateways` is refused: `NetworkArgs::validate` in
/// `freenet-libp2p-bevy-core` says it is not implemented, so every app would exit at start.
pub struct BrpNetwork {
    network: Network,
    token: String,
    deadlines: Deadlines,
    run_dir: PathBuf,
    orchestrator: Option<FakeNetworkServer>,
    _helper_nodes: Vec<FreenetNodeDir>,
}

impl BrpNetwork {
    /// # Errors
    /// Returns an error if the run directory cannot be made, the fake server cannot start,
    /// the lobby contract cannot be ensured, or the selected network is direct-dial.
    pub async fn prepare() -> Result<Self, String> {
        let network = Network::from_env();
        let stamp = unix_secs();
        let run_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(".local-run")
            .join(format!("brp-{stamp}"));
        std::fs::create_dir_all(&run_dir).map_err(|e| format!("run dir: {e}"))?;
        let deadlines = Deadlines::for_network(network);
        match network {
            Network::Fake => {
                let server =
                    FakeNetworkServer::start_with(Box::new(LobbyKit::default())).await?;
                Ok(Self {
                    network,
                    token: format!("brp-{}", unix_nanos()),
                    deadlines,
                    run_dir,
                    orchestrator: Some(server),
                    _helper_nodes: Vec::new(),
                })
            }
            Network::Mainnet => {
                let token = LobbySetup::Ensured.lobby_token(stamp).to_string();
                let helpers = check_or_deploy(&LobbyToken::from(token.as_str()))
                    .await
                    .map_err(|error| format!("lobby contract not ensured: {error:?}"))?;
                Ok(Self {
                    network,
                    token,
                    deadlines,
                    run_dir,
                    orchestrator: None,
                    _helper_nodes: helpers,
                })
            }
            Network::LocalGateways => Err(
                "local-gateways is not implemented: NetworkArgs::validate rejects it, so no peer can start; use fake or mainnet"
                    .to_string(),
            ),
            Network::DirectDial => Err(
                "direct-dial is not supported by the BRP scenarios yet; use fake or mainnet"
                    .to_string(),
            ),
        }
    }

    #[must_use]
    pub const fn deadlines(&self) -> Deadlines {
        self.deadlines
    }

    /// The `--network` value the app parses for this run.
    #[must_use]
    pub const fn network_arg(&self) -> &'static str {
        match self.network {
            Network::Fake => "fake",
            Network::Mainnet => "mainnet",
            Network::LocalGateways => "local-gateways",
            Network::DirectDial => "direct-dial",
        }
    }

    /// The arguments one peer is launched with on this network.
    #[must_use]
    pub fn peer_args(&self, username: &str, window: (i32, i32)) -> Vec<String> {
        let node_dir = self.run_dir.join(format!("{username}-freenet"));
        let mut args = vec![
            "--username".to_string(),
            username.to_string(),
            "--token".to_string(),
            self.token.clone(),
            "--network".to_string(),
            self.network_arg().to_string(),
            "--window-position".to_string(),
            format!("{},{}", window.0, window.1),
            "--freenet-node-dir".to_string(),
            node_dir.display().to_string(),
        ];
        if let Some(server) = &self.orchestrator {
            args.push("--orchestrator-port".to_string());
            args.push(server.0.to_string());
        }
        args
    }
}

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

fn unix_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos())
}
