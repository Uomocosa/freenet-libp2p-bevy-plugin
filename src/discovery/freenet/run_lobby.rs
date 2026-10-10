use std::time::{Duration, Instant};

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Receiver;

use crate::discovery;
use crate::net_id;
use discovery::freenet::basic::constants;

pub async fn run_lobby(
    mut lobby_client: discovery::freenet::LobbyClient,
    me: net_id::PeerId,
    capacity: u16,
    timing: discovery::Timing,
    mut target: Receiver<Option<discovery::state_machine::PublishTarget>>,
    lobbies: UnboundedSender<discovery::Lobby>,
) {
    let mut tick = tokio::time::interval(timing.tick.max(Duration::from_secs(1)));
    let mut last_sent: Option<discovery::Lobby> = None;
    let mut last_refresh: Option<Instant> = None;
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
        let mut lobby = discovery::freenet::poll(&mut lobby_client).await.ok();
        if is_due(last_refresh, now, timing.lobby) {
            let timeout = constants::REQUEST_TIMEOUT;
            let refresh = discovery::freenet::refresh(&mut lobby_client);
            if let Ok(Ok(fresh)) = tokio::time::timeout(timeout, refresh).await {
                lobby = Some(fresh);
            }
            last_refresh = Some(now);
        }
        if let Some(lobby) = lobby
            && last_sent.as_ref() != Some(&lobby)
        {
            last_sent = Some(lobby.clone());
            if lobbies.send(lobby).is_err() {
                return;
            }
        }
        let current = target.borrow_and_update().clone();
        if is_due(last_publish, now, timing.republish)
            && let Some(current) = current
        {
            let presence = discovery::Presence {
                addrs: current.addrs,
                updated_at: discovery::UnixTime::now(),
            };
            let _ = discovery::freenet::publish_presence(
                &mut lobby_client,
                &current.room,
                &me,
                presence,
                capacity,
            );
            last_publish = Some(now);
        }
    }
}

// needed helper: true when `every` has elapsed since `last` (or it never ran)
fn is_due(last: Option<Instant>, now: Instant, every: Duration) -> bool {
    last.is_none_or(|last| now.duration_since(last) >= every)
}

// no test_usage necessary
