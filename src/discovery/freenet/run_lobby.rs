use std::time::{Duration, Instant};

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Receiver;

use crate::discovery;
use discovery::freenet::basic::constants;

pub async fn run_lobby(
    freenet_config: &discovery::freenet::Config,
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
        let mut lobby = discovery::freenet::poll(&mut subscription).await.ok();
        if is_due(last_bridge, now, constants::RESUBSCRIBE) {
            let _ = discovery::freenet::bridge_subscription(&mut subscription);
            last_bridge = Some(now);
        }
        if is_due(last_refresh, now, constants::LOBBY_REFRESH) {
            let timeout = constants::REQUEST_TIMEOUT;
            let refresh = discovery::freenet::refresh(&mut subscription);
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
        if is_due(
            last_publish,
            now,
            freenet_config.discovery.lease.renew_every,
        ) {
            for current in current {
                let heartbeat = discovery::Heartbeat {
                    addrs: current.addrs,
                    updated_at: discovery::UnixTime::now(),
                    display_name: Some(current.display_name),
                    presence: current.presence,
                };
                let room = current.room.clone();
                match discovery::freenet::publish_heartbeat(
                    &mut subscription,
                    &current.room,
                    &freenet_config.me,
                    heartbeat,
                    freenet_config.discovery.room_size,
                    current.created_at,
                ) {
                    Ok(()) => {
                        tracing::info!(target: "discovery", ?room, "lobby heartbeat published");
                    }
                    Err(e) => {
                        tracing::warn!(target: "discovery", ?room, error = %e, "lobby heartbeat publish failed");
                    }
                }
            }
            last_publish = Some(now);
        }
    }
}

// needed helper: true when `every` has elapsed since `last` (or it never ran)
fn is_due(last: Option<Instant>, now: Instant, every: Duration) -> bool {
    last.is_none_or(|last| now.duration_since(last) >= every)
}

// no test_usage necessary
