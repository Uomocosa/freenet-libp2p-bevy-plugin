use std::time::{Duration, Instant};

use async_trait::async_trait;
use freenet_libp2p_bevy_plugin::{identity, testing};
use serde::Deserialize;

const POLL_INTERVAL: Duration = Duration::from_secs(10);
const DESCRIPTION_LIMIT: usize = 140;

pub struct GitHubCheckpointBoard {
    client: reqwest::Client,
    owner: String,
    repo: String,
    sha: String,
    prefix: String,
    cached: tokio::sync::Mutex<Option<CachedStatuses>>,
}

struct CachedStatuses {
    at: Instant,
    statuses: Vec<BoardStatus>,
}

#[derive(Debug, Clone, Deserialize)]
struct BoardStatus {
    context: String,
    #[serde(default)]
    description: Option<String>,
    state: String,
}

impl GitHubCheckpointBoard {
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

    pub fn from_env() -> Result<Self, String> {
        let repository = std::env::var("GITHUB_REPOSITORY").unwrap_or_default();
        let (owner, repo) = repository
            .split_once('/')
            .filter(|(owner, repo)| !owner.trim().is_empty() && !repo.trim().is_empty())
            .ok_or_else(|| "GITHUB_REPOSITORY must be owner/repo".to_string())?;
        let sha = std::env::var("GITHUB_SHA").unwrap_or_default();
        if sha.trim().is_empty() {
            return Err("GITHUB_SHA is missing".to_string());
        }
        std::env::var("GITHUB_TOKEN").map_err(|_| "GITHUB_TOKEN is missing".to_string())?;
        let prefix = if let Some(prefix) = std::env::var("E2E_BOARD_RUN_ID")
            .ok()
            .map(|raw| raw.trim().to_string())
            .filter(|raw| !raw.is_empty())
        {
            prefix
        } else {
            let run = std::env::var("GITHUB_RUN_ID").unwrap_or_default();
            if run.trim().is_empty() {
                return Err("GITHUB_RUN_ID is missing".to_string());
            }
            let attempt = std::env::var("GITHUB_RUN_ATTEMPT").unwrap_or_default();
            let attempt = attempt.trim();
            if attempt.is_empty() {
                run.trim().to_string()
            } else {
                format!("{}-{attempt}", run.trim())
            }
        };
        Ok(Self::new(
            owner.to_string(),
            repo.to_string(),
            sha.trim().to_string(),
            prefix,
        ))
    }

    fn peer_context(prefix: &str, name: &str, index: usize) -> String {
        format!("{prefix}/peer/{name}/{index}")
    }

    fn checkpoint_context(prefix: &str, name: &str) -> String {
        format!("{prefix}/checkpoint/{name}")
    }

    fn peer_descriptions(peer: &identity::Peer) -> Vec<String> {
        if peer.addrs.is_empty() {
            return vec![format!("peer-id:{}", peer.id)];
        }
        peer.addrs
            .iter()
            .map(|addr| format!("{addr}/p2p/{}", peer.id))
            .collect()
    }

    fn peer_from_statuses(
        name: &str,
        prefix: &str,
        statuses: &[BoardStatus],
    ) -> Option<identity::Peer> {
        let mut indexed: Vec<(usize, &BoardStatus)> = statuses
            .iter()
            .filter_map(|status| {
                status
                    .context
                    .strip_prefix(&format!("{prefix}/peer/{name}/"))?
                    .parse::<usize>()
                    .ok()
                    .map(|index| (index, status))
            })
            .collect();
        indexed.sort_by_key(|(index, _)| *index);
        let mut id: Option<identity::PeerId> = None;
        let mut addrs: Vec<identity::PeerAddr> = Vec::new();
        for (_, status) in indexed {
            let description = status.description.as_deref().unwrap_or("");
            if let Ok(peer) = identity::Peer::try_from(description) {
                let suffix = format!("/p2p/{}", peer.id);
                if id.is_none() {
                    id = Some(peer.id.clone());
                }
                for addr in peer.addrs {
                    let text = addr.to_string();
                    let bare = text.strip_suffix(suffix.as_str()).unwrap_or(text.as_str());
                    addrs.push(identity::PeerAddr::from(bare));
                }
            } else if let Some(raw) = description.strip_prefix("peer-id:")
                && !raw.is_empty()
                && id.is_none()
            {
                id = Some(identity::PeerId::from(raw));
            }
        }
        id.map(|id| identity::Peer { id, addrs })
    }

    fn checkpoint_done(name: &str, prefix: &str, statuses: &[BoardStatus]) -> bool {
        let context = Self::checkpoint_context(prefix, name);
        statuses
            .iter()
            .any(|status| status.context == context && status.state == "success")
    }

    fn commit_url(&self) -> String {
        format!(
            "https://api.github.com/repos/{}/{}/commits/{}",
            self.owner, self.repo, self.sha
        )
    }

    fn token() -> Result<String, String> {
        std::env::var("GITHUB_TOKEN").map_err(|_| "GITHUB_TOKEN is missing".to_string())
    }

    async fn listed_statuses(&self) -> Result<Vec<BoardStatus>, String> {
        {
            let cached = self.cached.lock().await;
            if let Some(cached) = cached.as_ref()
                && cached.at.elapsed() < POLL_INTERVAL
            {
                return Ok(cached.statuses.clone());
            }
        }
        let url = format!("{}/status?per_page=100", self.commit_url());
        let response = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .bearer_auth(Self::token()?)
            .send()
            .await
            .map_err(|error| format!("list statuses: {error}"))?;
        if !response.status().is_success() {
            return Err(format!("list statuses: {}", response.status()));
        }
        let combined: serde_json::Value = response
            .json()
            .await
            .map_err(|error| format!("decode statuses: {error}"))?;
        let statuses: Vec<BoardStatus> = combined
            .get("statuses")
            .cloned()
            .map(serde_json::from_value)
            .transpose()
            .map_err(|error| format!("decode statuses: {error}"))?
            .unwrap_or_default();
        *self.cached.lock().await = Some(CachedStatuses {
            at: Instant::now(),
            statuses: statuses.clone(),
        });
        Ok(statuses)
    }

    async fn post_status(&self, context: &str, description: &str) -> Result<(), String> {
        let url = format!("{}/statuses", self.commit_url());
        let response = self
            .client
            .post(&url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .bearer_auth(Self::token()?)
            .json(&serde_json::json!({
                "state": "success",
                "context": context,
                "description": description,
            }))
            .send()
            .await
            .map_err(|error| format!("post status {context}: {error}"))?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("post status {context}: {}", response.status()))
        }
    }
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
        for (index, description) in Self::peer_descriptions(peer).iter().enumerate() {
            let context = Self::peer_context(&self.prefix, name, index);
            self.post_status(&context, description).await?;
        }
        Ok(())
    }

    async fn find_peer(&self, name: &str) -> Result<Option<identity::Peer>, String> {
        let statuses = self.listed_statuses().await?;
        Ok(Self::peer_from_statuses(name, &self.prefix, &statuses))
    }
}

#[cfg(test)]
mod tests {
    use super::{BoardStatus, DESCRIPTION_LIMIT, GitHubCheckpointBoard};
    use freenet_libp2p_bevy_plugin::identity;

    fn peer_id() -> String {
        format!("12D3Koo{}", "a".repeat(45))
    }

    fn peer() -> identity::Peer {
        identity::Peer {
            id: identity::PeerId::from(peer_id()),
            addrs: vec![
                identity::PeerAddr::from("/ip4/192.168.1.20/tcp/51234"),
                identity::PeerAddr::from("/ip4/192.168.1.20/udp/51235/quic-v1"),
            ],
        }
    }

    fn status(context: &str, description: &str) -> BoardStatus {
        BoardStatus {
            context: context.to_string(),
            description: Some(description.to_string()),
            state: "success".to_string(),
        }
    }

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
        assert_eq!(peer_id().len(), 52);
        let descriptions = GitHubCheckpointBoard::peer_descriptions(&peer());
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
        let original = peer();
        let descriptions = GitHubCheckpointBoard::peer_descriptions(&original);
        let statuses: Vec<BoardStatus> = descriptions
            .iter()
            .enumerate()
            .map(|(index, description)| {
                status(
                    &GitHubCheckpointBoard::peer_context(prefix, "alice", index),
                    description,
                )
            })
            .collect();
        let found =
            GitHubCheckpointBoard::peer_from_statuses("alice", prefix, &statuses).expect("peer");
        assert_eq!(found, original);
        assert!(GitHubCheckpointBoard::peer_from_statuses("mallory", prefix, &statuses).is_none());
        let pending: Vec<BoardStatus> = statuses
            .into_iter()
            .map(|mut status| {
                status.state = "pending".to_string();
                status
            })
            .collect();
        assert!(!GitHubCheckpointBoard::checkpoint_done(
            "done", prefix, &pending
        ));
        let done = vec![status(
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
            id: identity::PeerId::from(peer_id()),
            addrs: Vec::new(),
        };
        let descriptions = GitHubCheckpointBoard::peer_descriptions(&id_only);
        assert_eq!(descriptions.len(), 1);
        assert!(descriptions[0].len() <= DESCRIPTION_LIMIT);
        let statuses = vec![status(
            &GitHubCheckpointBoard::peer_context(prefix, "bob", 0),
            &descriptions[0],
        )];
        let found =
            GitHubCheckpointBoard::peer_from_statuses("bob", prefix, &statuses).expect("peer");
        assert_eq!(found.id.as_str(), peer_id());
        assert_eq!(found.addrs.len(), 0);
    }

    #[test]
    fn publish_then_find_returns_the_same_peer() {
        let prefix = "run-1";
        let original = peer();
        let descriptions = GitHubCheckpointBoard::peer_descriptions(&original);
        let statuses: Vec<BoardStatus> = descriptions
            .iter()
            .enumerate()
            .map(|(index, description)| {
                status(
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
        for description in GitHubCheckpointBoard::peer_descriptions(&peer()) {
            assert!(description.len() <= DESCRIPTION_LIMIT, "{description}");
        }
        let id_only = identity::Peer {
            id: identity::PeerId::from(peer_id()),
            addrs: Vec::new(),
        };
        for description in GitHubCheckpointBoard::peer_descriptions(&id_only) {
            assert!(description.len() <= DESCRIPTION_LIMIT, "{description}");
        }
    }

    #[test]
    fn missing_config_is_an_error() {
        // SAFETY: nextest runs each test in its own process, so mutating the
        // environment here cannot race with another test.
        unsafe {
            std::env::remove_var("GITHUB_REPOSITORY");
            std::env::remove_var("GITHUB_SHA");
            std::env::remove_var("GITHUB_TOKEN");
            std::env::remove_var("GITHUB_RUN_ID");
            std::env::remove_var("E2E_BOARD_RUN_ID");
        }
        assert!(GitHubCheckpointBoard::from_env().is_err());
    }
}
