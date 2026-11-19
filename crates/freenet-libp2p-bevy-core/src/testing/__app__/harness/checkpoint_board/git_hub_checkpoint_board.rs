use async_trait::async_trait;
use atomic_delegate_macros::atomic_delegates;

use crate::identity;
use crate::testing;

pub struct GitHubCheckpointBoard {
    pub(crate) client: reqwest::Client,
    pub(crate) owner: String,
    pub(crate) repo: String,
    pub(crate) sha: String,
    pub(crate) prefix: String,
    pub(crate) cached: tokio::sync::Mutex<Option<testing::harness::CachedStatuses>>,
}

impl GitHubCheckpointBoard {
    #[must_use]
    pub fn new(owner: String, repo: String, sha: String, prefix: String) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("freenet-libp2p-e2e-board")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            client,
            owner,
            repo,
            sha,
            prefix,
            cached: tokio::sync::Mutex::new(None),
        }
    }
}

#[atomic_delegates]
impl GitHubCheckpointBoard {
    pub fn try_from_env() -> Result<Self, String> {}
}

#[async_trait]
impl testing::traits::CheckpointBoard for GitHubCheckpointBoard {
    async fn mark_completed(&self, checkpoint: &testing::ScriptCheckpoint) -> Result<(), String> {
        let context = Self::checkpoint_context(&self.prefix, &checkpoint.to_string());
        let description = format!("checkpoint {checkpoint} completed");
        self.post_status(&context, &description).await
    }

    async fn has_completed(&self, checkpoint: &testing::ScriptCheckpoint) -> Result<bool, String> {
        let statuses = self.listed_statuses().await?;
        Ok(Self::checkpoint_done(
            &checkpoint.to_string(),
            &self.prefix,
            &statuses,
        ))
    }

    async fn publish_peer(&self, name: &str, peer: &identity::Peer) -> Result<(), String> {
        let descriptions = Self::peer_descriptions(peer);
        self.post_peer_descriptions(name, &descriptions).await
    }

    async fn find_peer(&self, name: &str) -> Result<Option<identity::Peer>, String> {
        let statuses = self.listed_statuses().await?;
        Ok(Self::peer_from_statuses(name, &self.prefix, &statuses))
    }
}

#[rustfmt::skip]
impl GitHubCheckpointBoard {
    pub(crate) fn peer_context(prefix: &str, name: &str, index: usize) -> String { format!("{prefix}/peer/{name}/{index}") }
    pub(crate) fn checkpoint_context(prefix: &str, name: &str) -> String { format!("{prefix}/checkpoint/{name}") }
    pub(crate) fn token() -> Result<String, String> { std::env::var("GITHUB_TOKEN").map_err(|_| "GITHUB_TOKEN is missing".to_string()) }
}

impl GitHubCheckpointBoard {
    pub(crate) fn peer_descriptions(peer: &identity::Peer) -> Vec<String> {
        if peer.addrs.is_empty() {
            return vec![format!("peer-id:{}", peer.id)];
        }
        peer.addrs
            .iter()
            .map(|addr| format!("{addr}/p2p/{}", peer.id))
            .collect()
    }

    pub(crate) fn checkpoint_done(
        name: &str,
        prefix: &str,
        statuses: &[testing::harness::BoardStatus],
    ) -> bool {
        let context = Self::checkpoint_context(prefix, name);
        statuses.iter().any(|status| status.is_success(&context))
    }
}

#[atomic_delegates]
impl GitHubCheckpointBoard {
    pub(crate) fn commit_url(&self) -> String {}
    pub(crate) fn peer_from_statuses(
        name: &str,
        prefix: &str,
        statuses: &[testing::harness::BoardStatus],
    ) -> Option<identity::Peer> {
    }
    pub(crate) async fn listed_statuses(
        &self,
    ) -> Result<Vec<testing::harness::BoardStatus>, String> {
    }
    pub(crate) async fn post_status(&self, context: &str, description: &str) -> Result<(), String> {
    }
    pub(crate) async fn post_peer_descriptions(
        &self,
        name: &str,
        descriptions: &[String],
    ) -> Result<(), String> {
    }
}

#[cfg(test)]
mod tests {
    use super::GitHubCheckpointBoard;
    use crate::identity;
    use crate::testing;

    const DESCRIPTION_LIMIT: usize = 140;

    #[test]
    fn test_usage() {
        assert_eq!(
            GitHubCheckpointBoard::peer_context("run-1", "alice", 0),
            "run-1/peer/alice/0"
        );
        assert_eq!(
            GitHubCheckpointBoard::checkpoint_context("run-1", "done"),
            "run-1/checkpoint/done"
        );
        let board = GitHubCheckpointBoard::new(
            "owner".to_string(),
            "repo".to_string(),
            "sha".to_string(),
            "run-1".to_string(),
        );
        assert!(board.commit_url().contains("owner/repo/commits/sha"));
    }

    #[test]
    fn run_prefixes_keep_attempts_apart() {
        assert_ne!(
            GitHubCheckpointBoard::peer_context("run-1", "alice", 0),
            GitHubCheckpointBoard::peer_context("run-2", "alice", 0)
        );
        assert_ne!(
            GitHubCheckpointBoard::checkpoint_context("run-1", "done"),
            GitHubCheckpointBoard::checkpoint_context("run-2", "done")
        );
    }

    #[test]
    fn one_address_fits_but_two_do_not() {
        assert_eq!(testing::long_peer_id().len(), 52);
        let descriptions = GitHubCheckpointBoard::peer_descriptions(&testing::long_two_addr_peer());
        assert_eq!(descriptions.len(), 2);
        for description in &descriptions {
            assert!(description.len() <= DESCRIPTION_LIMIT, "{description}");
        }
        let joined = descriptions.join(" ");
        assert!(joined.len() > DESCRIPTION_LIMIT, "{joined}");
    }

    #[test]
    fn peers_reassemble_from_their_contexts() {
        let prefix = "run-1";
        let original = testing::long_two_addr_peer();
        let descriptions = GitHubCheckpointBoard::peer_descriptions(&original);
        let statuses: Vec<testing::harness::BoardStatus> = descriptions
            .iter()
            .enumerate()
            .map(|(index, description)| {
                testing::board_status(
                    &GitHubCheckpointBoard::peer_context(prefix, "alice", index),
                    description,
                )
            })
            .collect();
        let found =
            GitHubCheckpointBoard::peer_from_statuses("alice", prefix, &statuses).expect("peer");
        assert_eq!(found, original);
        assert!(GitHubCheckpointBoard::peer_from_statuses("mallory", prefix, &statuses).is_none());
        let pending: Vec<testing::harness::BoardStatus> = statuses
            .into_iter()
            .map(|mut status| {
                status.state = "pending".to_string();
                status
            })
            .collect();
        assert!(!GitHubCheckpointBoard::checkpoint_done(
            "done", prefix, &pending
        ));
        let done = vec![testing::board_status(
            &GitHubCheckpointBoard::checkpoint_context(prefix, "done"),
            "checkpoint done completed",
        )];
        assert!(GitHubCheckpointBoard::checkpoint_done(
            "done", prefix, &done
        ));
    }

    #[test]
    fn id_only_peers_resolve_their_name() {
        let prefix = "run-1";
        let id_only = identity::Peer {
            id: identity::PeerId::from(testing::long_peer_id()),
            addrs: Vec::new(),
        };
        let descriptions = GitHubCheckpointBoard::peer_descriptions(&id_only);
        assert_eq!(descriptions.len(), 1);
        assert!(descriptions[0].len() <= DESCRIPTION_LIMIT);
        let statuses = vec![testing::board_status(
            &GitHubCheckpointBoard::peer_context(prefix, "bob", 0),
            &descriptions[0],
        )];
        let found =
            GitHubCheckpointBoard::peer_from_statuses("bob", prefix, &statuses).expect("peer");
        assert_eq!(found.id.as_str(), testing::long_peer_id());
        assert_eq!(found.addrs.len(), 0);
    }

    #[test]
    fn publish_then_find_returns_the_same_peer() {
        let prefix = "run-1";
        let original = testing::long_two_addr_peer();
        let descriptions = GitHubCheckpointBoard::peer_descriptions(&original);
        let statuses: Vec<testing::harness::BoardStatus> = descriptions
            .iter()
            .enumerate()
            .map(|(index, description)| {
                testing::board_status(
                    &GitHubCheckpointBoard::peer_context(prefix, "alice", index),
                    description,
                )
            })
            .collect();
        let found =
            GitHubCheckpointBoard::peer_from_statuses("alice", prefix, &statuses).expect("peer");
        assert_eq!(found, original);
    }

    #[test]
    fn descriptions_stay_within_the_limit() {
        for description in GitHubCheckpointBoard::peer_descriptions(&testing::long_two_addr_peer())
        {
            assert!(description.len() <= DESCRIPTION_LIMIT, "{description}");
        }
        let id_only = identity::Peer {
            id: identity::PeerId::from(testing::long_peer_id()),
            addrs: Vec::new(),
        };
        for description in GitHubCheckpointBoard::peer_descriptions(&id_only) {
            assert!(description.len() <= DESCRIPTION_LIMIT, "{description}");
        }
    }

    #[test]
    fn missing_config_is_an_error() {
        unsafe {
            std::env::remove_var("GITHUB_REPOSITORY");
            std::env::remove_var("GITHUB_SHA");
            std::env::remove_var("GITHUB_TOKEN");
            std::env::remove_var("GITHUB_RUN_ID");
            std::env::remove_var("E2E_BOARD_RUN_ID");
        }
        assert!(GitHubCheckpointBoard::try_from_env().is_err());
    }
}
