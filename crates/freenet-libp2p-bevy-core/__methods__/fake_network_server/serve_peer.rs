use std::sync::Arc;

use tokio::io::AsyncReadExt;

use crate::testing;
use crate::time;

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
    let hello_frames = {
        let mut guard = shared.lock().await;
        guard.connect_peer(&hello.peer, &hello.game, frame_tx.clone());
        let hello_frames = guard
            .kit
            .on_hello(&hello.peer, &hello.game, time::UnixTime::now());
        drop(guard);
        hello_frames
    };
    for bytes in hello_frames {
        frame_tx
            .send(testing::FakeWireServerFrame::Extension(bytes))
            .ok();
    }
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
        let (events, fan_outs) = {
            let mut guard = shared.lock().await;
            let now = time::UnixTime::now();
            let fan_outs = match &frame {
                testing::FakeWireClientFrame::Extension(bytes) => {
                    guard.route_extension(&peer_id, bytes.clone(), now)
                }
                _ => Vec::new(),
            };
            let events = guard.handle_client_frame(&peer_id, frame);
            drop(guard);
            (events, fan_outs)
        };
        task.send_routed_events(&shared, events).await;
        task.send_fan_out(&shared, &hello.game, fan_outs).await;
    }
    let (leftovers, disconnect_fan_outs) = {
        let mut guard = shared.lock().await;
        guard.remove_connection(&peer_id, time::UnixTime::now())
    };
    task.send_routed_events(&shared, leftovers).await;
    task.send_fan_out(&shared, &hello.game, disconnect_fan_outs)
        .await;
}
#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::io::AsyncWriteExt;

    use super::serve_peer;
    use crate::identity;
    use crate::testing;

    #[tokio::test]
    async fn test_usage() {
        let task = testing::FakeNetworkServer::start().await.expect("server");
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let port = listener.local_addr().expect("port").port();
        let mut client = tokio::net::TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .expect("connect");
        let (stream, _) = listener.accept().await.expect("accept");
        let shared = Arc::new(tokio::sync::Mutex::new(
            testing::orchestrator_shared::OrchestratorShared::default(),
        ));
        let peer = identity::Peer {
            id: identity::PeerId::from("peer"),
            addrs: Vec::new(),
        };
        let hello = testing::FakeWireHello {
            peer: peer.clone(),
            game: identity::GameId::from("game"),
        };
        let body = bincode::serialize(&hello).expect("encode");
        client
            .write_u32(u32::try_from(body.len()).expect("length"))
            .await
            .expect("length");
        client.write_all(&body).await.expect("body");

        let serving = tokio::spawn(serve_peer(task, stream, shared.clone()));
        let connected = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                if shared.lock().await.senders.contains_key(&peer.id) {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
        })
        .await;
        serving.abort();
        assert!(connected.is_ok());
        assert!(shared.lock().await.peer_games.contains_key(&peer.id));
    }
}
