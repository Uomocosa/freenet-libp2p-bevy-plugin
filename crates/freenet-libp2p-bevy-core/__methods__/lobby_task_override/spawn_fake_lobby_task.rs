use crate::discovery;
use crate::freenet;
use crate::identity;
use crate::testing;

#[must_use]
pub fn spawn_fake_lobby_task(
    task_override: discovery::LobbyTaskOverride,
    channels: discovery::LobbyTaskChannels,
    game: identity::GameId,
) -> tokio::task::JoinHandle<()> {
    let Some(port) = task_override.orchestrator_port else {
        return tokio::spawn(async move {
            let _ = (channels, game);
            std::future::pending::<()>().await;
        });
    };
    tokio::spawn(async move {
        let discovery::LobbyTaskChannels {
            mut heartbeat_targets,
            lobbies,
            freenet_peers,
        } = channels;
        freenet_peers
            .send(vec![freenet::FreenetPeer {
                id: freenet::FreenetPeerId::from("fake-gateway"),
                addr: std::net::SocketAddr::from(([127, 0, 0, 1], 1)),
            }])
            .ok();
        let framing = testing::FakeWireFraming;
        let Ok(stream) = tokio::net::TcpStream::connect(format!("127.0.0.1:{port}")).await else {
            tracing::error!(target: "testing", "fake lobby: orchestrator unreachable");
            return;
        };
        let (reader, mut writer) = stream.into_split();
        let hello = testing::FakeWireHello {
            peer: task_override.peer.clone(),
            game: game.clone(),
        };
        if !framing.write_frame(&mut writer, &hello).await {
            tracing::error!(target: "testing", "fake lobby: hello write failed");
            return;
        }
        let mut server_frames = framing.spawn_server_frame_reader(reader);
        let mut republish = tokio::time::interval(
            discovery::HolderHeartbeatIntervals::default().interval_between_heartbeats,
        );
        loop {
            tokio::select! {
                changed = heartbeat_targets.changed() => {
                    if changed.is_err() {
                        break;
                    }
                    let targets = heartbeat_targets.borrow_and_update().clone();
                    let frame = testing::FakeWireClientFrame::Extension(
                        bincode::serialize(&discovery::LobbyWire::HeartbeatTargets(targets))
                            .unwrap_or_default(),
                    );
                    if !framing.write_frame(&mut writer, &frame).await {
                        tracing::error!(target: "testing", "fake lobby: heartbeat write failed");
                        break;
                    }
                }
                _ = republish.tick() => {
                    let targets = heartbeat_targets.borrow_and_update().clone();
                    let frame = testing::FakeWireClientFrame::Extension(
                        bincode::serialize(&discovery::LobbyWire::HeartbeatTargets(targets))
                            .unwrap_or_default(),
                    );
                    if !framing.write_frame(&mut writer, &frame).await {
                        tracing::error!(target: "testing", "fake lobby: heartbeat write failed");
                        break;
                    }
                }
                incoming = server_frames.recv() => {
                    let Some(frame) = incoming else {
                        tracing::error!(target: "testing", "fake lobby: server stream ended");
                        break;
                    };
                    match frame {
                        testing::FakeWireServerFrame::Extension(bytes) => {
                            let Ok(discovery::LobbyWire::Snapshot(lobby)) =
                                bincode::deserialize::<discovery::LobbyWire>(&bytes)
                            else {
                                continue;
                            };
                            if lobbies.send(lobby).is_err() {
                                break;
                            }
                        }
                        testing::FakeWireServerFrame::Libp2pEvent(_)
                        | testing::FakeWireServerFrame::FreenetPeers(_) => {}
                    }
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::spawn_fake_lobby_task;
    use crate::discovery;
    use crate::identity;

    #[tokio::test]
    async fn test_usage() {
        let task_override = discovery::LobbyTaskOverride {
            orchestrator_port: Some(1),
            peer: identity::Peer {
                id: identity::PeerId::from("peer"),
                addrs: Vec::new(),
            },
        };
        let (_target_tx, target_rx) = tokio::sync::watch::channel(Vec::new());
        let (lobbies_tx, _lobbies_rx) = tokio::sync::mpsc::unbounded_channel();
        let (peers_tx, _peers_rx) = tokio::sync::mpsc::unbounded_channel();
        let handle = spawn_fake_lobby_task(
            task_override,
            discovery::LobbyTaskChannels {
                heartbeat_targets: target_rx,
                lobbies: lobbies_tx,
                freenet_peers: peers_tx,
            },
            identity::GameId::from("game"),
        );
        handle.abort();
    }
}
