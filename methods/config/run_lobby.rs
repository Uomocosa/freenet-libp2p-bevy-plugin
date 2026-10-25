use std::time::Instant;

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Receiver;

use crate::discovery;
use crate::identity;
use discovery::freenet::basic::constants;

pub async fn run_lobby(
    state: &discovery::freenet::Config,
    mut subscription: discovery::freenet::LobbyContractSubscription,
    mut target: Receiver<Vec<discovery::state_machine::HeartbeatTarget>>,
    lobbies_tx: UnboundedSender<discovery::Lobby>,
) {
    let mut interval = tokio::time::interval(discovery::TICK);
    let mut last_sent: Option<discovery::Lobby> = None;
    let mut last_refresh: Option<Instant> = None;
    let mut last_bridge: Option<Instant> = None;
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
        if discovery::freenet::Config::is_due(last_bridge, now, constants::RESUBSCRIBE) {
            let _ = subscription.bridge_subscription();
            last_bridge = Some(now);
        }
        if discovery::freenet::Config::is_due(last_refresh, now, constants::LOBBY_REFRESH) {
            let timeout = constants::REQUEST_TIMEOUT;
            let refresh = subscription.refresh();
            if let Ok(Ok(fresh)) = tokio::time::timeout(timeout, refresh).await {
                lobby = Some(fresh);
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
        if discovery::freenet::Config::is_due(
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
                    updated_at: discovery::UnixTime::now(),
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
    use super::run_lobby;

    use crate::discovery;
    use crate::identity;

    #[tokio::test]
    async fn test_usage() {
        let config = discovery::freenet::Config {
            host: "127.0.0.1".to_string(),
            port: 0,
            me: identity::PeerId::from("me"),
            params: Vec::new(),
            discovery: discovery::Config::default(),
        };
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let subscription = discovery::freenet::LobbyContractSubscription {
            client: discovery::freenet::Client { write, read },
            key: freenet_stdlib::prelude::WrappedContract::new(
                std::sync::Arc::new(freenet_stdlib::prelude::ContractCode::from(vec![0; 4])),
                freenet_stdlib::prelude::Parameters::from(Vec::new()),
            )
            .key,
            lobby: discovery::Lobby::default(),
        };
        let (_target_tx, target_rx) = tokio::sync::watch::channel(Vec::new());
        let (lobbies_tx, _lobbies_rx) = tokio::sync::mpsc::unbounded_channel();
        let handle = tokio::spawn(async move {
            run_lobby(&config, subscription, target_rx, lobbies_tx).await;
        });
        handle.abort();
    }
}
