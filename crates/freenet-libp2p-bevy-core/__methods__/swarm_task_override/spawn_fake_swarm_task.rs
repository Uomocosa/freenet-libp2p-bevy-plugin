use std::collections::BTreeMap;

use crate::identity;
use crate::p2p;
use crate::testing;

#[must_use]
pub fn spawn_fake_swarm_task<Payload: p2p::traits::Message>(
    task_override: p2p::SwarmTaskOverride<Payload>,
    mut commands: tokio::sync::mpsc::UnboundedReceiver<p2p::Command<Payload>>,
    mut libp2p_commands: tokio::sync::mpsc::UnboundedReceiver<p2p::Libp2pCommand>,
    events: tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
    game: identity::GameId,
) -> tokio::task::JoinHandle<()> {
    let framing = testing::FakeWireFraming;
    tokio::spawn(async move {
        let Ok(stream) = tokio::net::TcpStream::connect(format!(
            "127.0.0.1:{}",
            task_override.orchestrator_port
        ))
        .await
        else {
            tracing::error!(target: "testing", "fake swarm: orchestrator unreachable");
            events
                .send(p2p::Event::Error(
                    "fake network: orchestrator unreachable".to_string(),
                ))
                .ok();
            return;
        };
        let (reader, mut writer) = stream.into_split();
        let hello = testing::FakeWireHello {
            peer: task_override.peer.clone(),
            game: game.clone(),
        };
        if !framing.write_frame(&mut writer, &hello).await {
            tracing::error!(target: "testing", "fake swarm: hello write failed");
            return;
        }
        events
            .send(p2p::Event::Libp2p(p2p::Libp2pEvent::Ready(
                task_override.peer.clone(),
            )))
            .ok();
        let mut rooms_by_topic: BTreeMap<identity::Topic, identity::RoomName> = BTreeMap::new();
        let mut server_frames = framing.spawn_server_frame_reader(reader);
        loop {
            tokio::select! {
                cmd = commands.recv() => {
                    let Some(cmd) = cmd else { break };
                    let Some(frame) = task_override.command_frame(&game, &mut rooms_by_topic, &events, cmd) else { continue };
                    if !framing.write_frame(&mut writer, &frame).await {
                        tracing::error!(target: "testing", "fake swarm: command write failed");
                        break;
                    }
                }
                cmd = libp2p_commands.recv() => {
                    let Some(command) = cmd else { break };
                    task_override.track_room_command(&command, &game, &mut rooms_by_topic);
                    let frame = testing::FakeWireClientFrame::Libp2pCommand(command);
                    if !framing.write_frame(&mut writer, &frame).await {
                        tracing::error!(target: "testing", "fake swarm: command write failed");
                        break;
                    }
                }
                incoming = server_frames.recv() => {
                    let Some(frame) = incoming else {
                        tracing::error!(target: "testing", "fake swarm: server stream ended");
                        break;
                    };
                    match frame {
                        testing::FakeWireServerFrame::Libp2pEvent(event) => {
                            task_override.handle_server_event(&rooms_by_topic, event, &events);
                        }
                        testing::FakeWireServerFrame::Lobby(_)
                        | testing::FakeWireServerFrame::FreenetPeers(_)
                        | testing::FakeWireServerFrame::Extension(_) => {}
                    }
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::spawn_fake_swarm_task;
    use crate::identity;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let task_override = p2p::SwarmTaskOverride::<u8> {
            orchestrator_port: 1,
            peer: identity::Peer {
                id: identity::PeerId::from("peer"),
                addrs: Vec::new(),
            },
            marker: std::marker::PhantomData,
        };
        let (_cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_libp2p_tx, libp2p_rx) = tokio::sync::mpsc::unbounded_channel();
        let (event_tx, _event_rx) = tokio::sync::mpsc::unbounded_channel();
        let handle = spawn_fake_swarm_task(
            task_override,
            cmd_rx,
            libp2p_rx,
            event_tx,
            identity::GameId::from("game"),
        );
        handle.abort();
    }
}
