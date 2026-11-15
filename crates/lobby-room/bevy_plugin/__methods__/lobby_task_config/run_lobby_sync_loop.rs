use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Receiver;
use tokio::time::Instant;

use crate::discovery;
use crate::freenet;
use crate::identity;
use crate::time;

pub async fn run_lobby_sync_loop(
    state: &discovery::LobbyTaskConfig,
    mut subscription: discovery::LobbyContractSubscription,
    mut target: Receiver<Vec<discovery::state_machine::HeartbeatTarget>>,
    lobbies_tx: UnboundedSender<discovery::Lobby>,
) {
    let mut interval = tokio::time::interval(discovery::TICK);
    let mut last_sent: Option<discovery::Lobby> = None;
    let mut last_refresh: Option<Instant> = None;
    let mut resubscribe_gate = discovery::ResubscribeGate::default();
    let mut last_publish: Option<Instant> = None;
    loop {
        tokio::select! {
            _ = interval.tick() => {}
            changed = target.changed() => {
                if changed.is_err() {
                    return;
                }
                last_publish = None;
            }
        }
        let now = Instant::now();
        let mut lobby = subscription.poll().await.ok();
        resubscribe_gate.heal_replica_split(&mut subscription, now);
        if discovery::LobbyTaskConfig::is_due(last_refresh, now, discovery::LOBBY_REFRESH) {
            let budget = freenet::REQUEST_TIMEOUT;
            let started_refresh = Instant::now();
            match tokio::time::timeout(budget, subscription.refresh()).await {
                Ok(Ok(fresh)) => {
                    tracing::debug!(
                        target: "discovery",
                        rooms = fresh.len(),
                        elapsed_ms = started_refresh.elapsed().as_millis(),
                        "lobby refresh ok"
                    );
                    lobby = Some(fresh);
                }
                Ok(Err(error)) => tracing::warn!(
                    target: "discovery",
                    ?error,
                    "lobby refresh failed"
                ),
                Err(_) => tracing::warn!(
                    target: "discovery",
                    budget_ms = budget.as_millis(),
                    "lobby refresh timed out"
                ),
            }
            last_refresh = Some(now);
        }
        if let Some(lobby) = lobby
            && last_sent.as_ref() != Some(&lobby)
        {
            tracing::info!(target: "discovery", rooms = lobby.len(), "lobby changed");
            last_sent = Some(lobby.clone());
            if lobbies_tx.send(lobby).is_err() {
                return;
            }
        }
        let current = target.borrow_and_update().clone();
        if discovery::LobbyTaskConfig::is_due(
            last_publish,
            now,
            state.discovery.lease.interval_between_heartbeats,
        ) {
            for current in current {
                let room_member = discovery::RoomMember {
                    peer: identity::Peer {
                        id: state.me.clone(),
                        addrs: current.addrs,
                    },
                    updated_at: time::UnixTime::now(),
                    display_name: Some(current.display_name),
                    membership: current.membership,
                };
                let room = current.room.clone();
                match subscription.publish_heartbeat(
                    &current.room,
                    &state.me,
                    room_member,
                    state.discovery.room_size,
                    current.created_at,
                ) {
                    Ok(()) => {
                        tracing::info!(target: "discovery", ?room, "lobby heartbeat published");
                    }
                    Err(e) => {
                        tracing::warn!(target: "discovery", ?room, error = ?e, "lobby heartbeat publish failed");
                    }
                }
            }
            last_publish = Some(now);
        }
    }
}
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
    use tokio_tungstenite::tungstenite::Message;

    use super::run_lobby_sync_loop;

    use crate::discovery;
    use crate::freenet;
    use crate::identity;

    #[tokio::test]
    async fn test_usage() {
        let config = discovery::LobbyTaskConfig {
            host: "127.0.0.1".to_string(),
            port: 0,
            me: identity::PeerId::from("me"),
            params: Vec::new(),
            discovery: discovery::Config::default(),
        };
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let subscription = discovery::LobbyContractSubscription {
            client: freenet::Client { write, read },
            key: freenet_stdlib::prelude::WrappedContract::new(
                std::sync::Arc::new(freenet_stdlib::prelude::ContractCode::from(vec![0; 4])),
                freenet_stdlib::prelude::Parameters::from(Vec::new()),
            )
            .key,
            lobby: discovery::Lobby::default(),
            has_unread_subscribe_answer: false,
        };
        let (_target_tx, target_rx) = tokio::sync::watch::channel(Vec::new());
        let (lobbies_tx, _lobbies_rx) = tokio::sync::mpsc::unbounded_channel();
        let handle = tokio::spawn(async move {
            run_lobby_sync_loop(&config, subscription, target_rx, lobbies_tx).await;
        });
        handle.abort();
    }

    #[tokio::test(start_paused = true)]
    async fn resubscribe_waits_out_the_backoff_while_unanswered() {
        let config = discovery::LobbyTaskConfig {
            host: "127.0.0.1".to_string(),
            port: 0,
            me: identity::PeerId::from("me"),
            params: Vec::new(),
            discovery: discovery::Config::default(),
        };
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let subscription = discovery::LobbyContractSubscription {
            client: freenet::Client { write, read },
            key: freenet_stdlib::prelude::WrappedContract::new(
                std::sync::Arc::new(freenet_stdlib::prelude::ContractCode::from(vec![0; 4])),
                freenet_stdlib::prelude::Parameters::from(Vec::new()),
            )
            .key,
            lobby: discovery::Lobby::default(),
            has_unread_subscribe_answer: false,
        };
        let (_target_tx, target_rx) = tokio::sync::watch::channel(Vec::new());
        let (lobbies_tx, _lobbies_rx) = tokio::sync::mpsc::unbounded_channel();
        let handle = tokio::spawn(async move {
            run_lobby_sync_loop(&config, subscription, target_rx, lobbies_tx).await;
        });
        tokio::task::yield_now().await;
        drive_virtual_seconds(9).await;
        assert_eq!(count_subscribes(&mut write_rx), 1);
        drive_virtual_seconds(36).await;
        assert_eq!(count_subscribes(&mut write_rx), 2);
        handle.abort();
    }

    async fn drive_virtual_seconds(seconds: u64) {
        for _ in 0..seconds.saturating_mul(10) {
            tokio::time::advance(Duration::from_millis(100)).await;
            tokio::task::yield_now().await;
        }
    }

    fn count_subscribes(write_rx: &mut tokio::sync::mpsc::UnboundedReceiver<Message>) -> u32 {
        let mut subscribes: u32 = 0;
        while let Ok(Message::Binary(bytes)) = write_rx.try_recv() {
            if let Ok(ClientRequest::ContractOp(ContractRequest::Subscribe { .. })) =
                bincode::deserialize::<ClientRequest<'_>>(&bytes)
            {
                subscribes = subscribes.saturating_add(1);
            }
        }
        subscribes
    }
}
