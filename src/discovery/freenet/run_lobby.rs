use std::time::{Duration, Instant};

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Receiver;

use crate::discovery;
use discovery::freenet::basic::constants;

pub async fn run_lobby(
    freenet_config: &discovery::freenet::FreenetConfig,
    mut lobby_contract: discovery::freenet::LobbyContract,
    mut target: Receiver<Option<discovery::state_machine::HeartbeatTarget>>,
    lobbies: UnboundedSender<discovery::Lobby>,
) {
    let mut tick = tokio::time::interval(freenet_config.timing.tick.max(Duration::from_secs(1)));
    let mut last_sent: Option<discovery::Lobby> = None;
    let mut last_refresh: Option<Instant> = None;
    let mut last_bridge: Option<Instant> = None;
    let mut last_publish: Option<Instant> = None;
    loop {
        tokio::select! {
            _ = tick.tick() => {}
            changed = target.changed() => {
                if changed.is_err() {
                    return;
                }
                last_publish = None;
            }
        }
        let now = Instant::now();
        let mut lobby = discovery::freenet::poll(&mut lobby_contract).await.ok();
        if is_due(last_bridge, now, freenet_config.timing.bridge) {
            let _ = discovery::freenet::bridge_subscription(&mut lobby_contract);
            last_bridge = Some(now);
        }
        if is_due(last_refresh, now, freenet_config.timing.lobby) {
            let timeout = constants::REQUEST_TIMEOUT;
            let refresh = discovery::freenet::refresh(&mut lobby_contract);
            if let Ok(Ok(fresh)) = tokio::time::timeout(timeout, refresh).await {
                lobby = Some(fresh);
            }
            last_refresh = Some(now);
        }
        if let Some(lobby) = lobby
            && last_sent.as_ref() != Some(&lobby)
        {
            tracing::info!(target: "discovery", rooms = lobby.len(), "lobby board changed");
            last_sent = Some(lobby.clone());
            if lobbies.send(lobby).is_err() {
                return;
            }
        }
        let current = target.borrow_and_update().clone();
        if is_due(last_publish, now, freenet_config.timing.republish)
            && let Some(current) = current
        {
            let heartbeat = discovery::Heartbeat {
                addrs: current.addrs,
                updated_at: discovery::UnixTime::now(),
                display_name: Some(current.display_name),
            };
            let room = current.room.clone();
            match discovery::freenet::publish_heartbeat(
                &mut lobby_contract,
                &current.room,
                &freenet_config.me,
                heartbeat,
                freenet_config.room_size,
            ) {
                Ok(()) => {
                    tracing::info!(target: "discovery", ?room, "lobby heartbeat published");
                }
                Err(e) => {
                    tracing::warn!(target: "discovery", ?room, error = %e, "lobby heartbeat publish failed");
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
