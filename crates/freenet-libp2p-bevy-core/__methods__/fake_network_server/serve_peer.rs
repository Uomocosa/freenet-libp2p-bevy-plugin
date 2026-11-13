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
    use crate::testing;

    #[tokio::test]
    async fn test_usage() {
        let _ = stringify!(serve_peer);
        let _ = testing::FakeNetworkServer::start().await.map(|_| ());
    }
}
