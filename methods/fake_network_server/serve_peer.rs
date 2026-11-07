use std::sync::Arc;

use tokio::io::AsyncReadExt;

use crate::discovery;
use crate::testing;

pub async fn serve_peer(
    task: testing::FakeNetworkServer,
    stream: tokio::net::TcpStream,
    shared: Arc<tokio::sync::Mutex<testing::orchestrator_shared::OrchestratorShared>>,
) {
    let framing = testing::FakeWireFraming;
    let (mut reader, mut writer) = stream.into_split();
    let (frame_tx, mut frame_rx) =
        tokio::sync::mpsc::unbounded_channel::<testing::FakeWireServerFrame>();
    tokio::spawn(async move {
        while let Some(frame) = frame_rx.recv().await {
            if !framing.write_frame(&mut writer, &frame).await {
                return;
            }
        }
    });
    let Ok(length) = reader.read_u32().await else {
        tracing::error!(target: "testing", "orchestrator: hello length read failed");
        return;
    };
    let length = usize::try_from(length).unwrap_or(0);
    let mut body = vec![0u8; length];
    if reader.read_exact(&mut body).await.is_err() {
        tracing::error!(target: "testing", "orchestrator: hello body read failed");
        return;
    }
    let Ok(hello) = bincode::deserialize::<testing::FakeWireHello>(&body) else {
        tracing::error!(target: "testing", "orchestrator: hello decode failed");
        return;
    };
    let current_lobby = {
        let mut guard = shared.lock().await;
        guard.hello_lobby(&hello.peer, &hello.game, frame_tx.clone())
    };
    frame_tx
        .send(testing::FakeWireServerFrame::Lobby(current_lobby))
        .ok();
    let peer_id = hello.peer.id.clone();
    loop {
        let Ok(length) = reader.read_u32().await else {
            tracing::error!(target: "testing", "orchestrator: frame length read failed");
            break;
        };
        let length = usize::try_from(length).unwrap_or(0);
        let mut body = vec![0u8; length];
        if reader.read_exact(&mut body).await.is_err() {
            tracing::error!(target: "testing", "orchestrator: frame body read failed");
            break;
        }
        let Ok(frame) = bincode::deserialize::<testing::FakeWireClientFrame>(&body) else {
            tracing::error!(target: "testing", "orchestrator: frame decode failed");
            break;
        };
        let (events, lobby) = {
            let mut guard = shared.lock().await;
            guard.handle_client_frame(&peer_id, frame, discovery::UnixTime::now())
        };
        task.send_routed_events(&shared, events).await;
        if let Some(lobby) = lobby {
            task.send_lobby_to_game_peers(&shared, &peer_id, lobby)
                .await;
        }
    }
    let leftovers = {
        let mut guard = shared.lock().await;
        guard.remove_connection(&peer_id)
    };
    task.send_routed_events(&shared, leftovers).await;
}

#[cfg(test)]
mod tests {
    use crate::testing;

    #[tokio::test]
    async fn test_usage() {
        let _ = stringify!(serve_peer);
        let _ = testing::FakeNetworkServer::start().await.map(|_| ());
    }
}
