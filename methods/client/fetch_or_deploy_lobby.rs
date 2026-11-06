use std::sync::Arc;
use std::time::Duration;

use freenet_stdlib::prelude::*;
use tracing::{info, warn};

use crate::discovery;
use discovery::Error;
use discovery::freenet;
use discovery::freenet::Client;
use discovery::freenet::DeployPolicy;
use discovery::freenet::FETCH_MAX_SENDS;
use discovery::freenet::FETCH_RESEND_AFTER;
use discovery::freenet::FETCH_SILENCE_LIMIT;
use discovery::freenet::RequestKind;

#[rustfmt::skip]
pub async fn fetch_or_deploy_lobby(
    state: &mut Client,
    contract_wasm: &[u8],
    params: &[u8],
    deploy: freenet::DeployPolicy,
    gateway_wait: Duration,
    seed_lobby: &discovery::Lobby,
) -> Result<(ContractKey, discovery::Lobby), discovery::Error> {
    state.wait_until_node_reachable(gateway_wait).await?;
    let code = Arc::new(ContractCode::from(contract_wasm.to_vec()));
    let wrapped = WrappedContract::new(code, Parameters::from(params.to_vec()));
    let key = wrapped.key;
    let instance_id = *key.id();
    info!(target: "discovery", %key, ?deploy, "lobby fetch start");
    let started = std::time::Instant::now();
    let mut sends: usize = 0;
    let fetch_all = async {
        loop {
            state.send_lobby_fetch(instance_id)?;
            sends = sends.saturating_add(1);
            if sends < FETCH_MAX_SENDS {
                match tokio::time::timeout(FETCH_RESEND_AFTER, state.await_outstanding_fetch())
                    .await
                {
                    Err(_) => {
                        info!(
                            target: "discovery",
                            sends,
                            elapsed_ms = started.elapsed().as_millis(),
                            "lobby fetch resent"
                        );
                    }
                    Ok(settled) => break settled,
                }
            } else {
                break state.await_outstanding_fetch().await;
            }
        }
    };
    let is_fetch_or_deploy = deploy == DeployPolicy::FetchOrDeploy;
    let (silent, raw_first) = if is_fetch_or_deploy {
        let timed = RequestKind::FetchLobby.run_with_timeout(fetch_all);
        match tokio::time::timeout(FETCH_SILENCE_LIMIT, timed).await {
            Err(_) | Ok(Err(Error::Timeout(RequestKind::FetchLobby))) => (true, Err(Error::Disconnected)),
            Ok(other) => (false, other),
        }
    } else { (false, RequestKind::FetchLobby.run_with_timeout(fetch_all).await) };
    let (deploy_unanswered, fetched) = if silent {
        let elapsed_ms = started.elapsed().as_millis();
        warn!(target: "discovery", sends, elapsed_ms, "lobby fetch silent, deploying");
        warn!(target: "discovery", elapsed_ms, sends, error = "Silent", "lobby fetch failed");
        (Some(sends), None)
    } else {
        let first = match raw_first {
            Err(Error::Timeout(RequestKind::FetchLobby)) => {
                info!(target: "discovery", %key, "lobby fetch timed out, awaiting the node's answer");
                state.await_outstanding_fetch().await
            }
            first => first,
        };
        let elapsed_ms = started.elapsed().as_millis();
        match first {
            Ok(lobby) => {
                info!(target: "discovery", elapsed_ms, sends, rooms = lobby.len(), "lobby fetch found");
                (None, Some(lobby))
            }
            Err(e) => {
                warn!(target: "discovery", elapsed_ms, sends, error = ?e, "lobby fetch failed");
                if is_fetch_or_deploy && matches!(e, Error::ContractNotFound) {
                    (Some(sends.saturating_sub(1)), None)
                } else {
                    return Err(e);
                }
            }
        }
    };
    let lobby = if let Some(unanswered_sends) = deploy_unanswered {
        info!(target: "discovery", %key, rooms = seed_lobby.len(), "lobby deploy start");
        let container = ContractContainer::from(ContractWasmAPIVersion::V1(wrapped));
        let deploy_skipped_stale = RequestKind::DeployLobby
            .run_with_timeout(state.deploy_contract(container, seed_lobby))
            .await
            .inspect_err(|e| warn!(target: "discovery", error = ?e, "lobby deploy failed"))?;
        let stale_not_founds = unanswered_sends.saturating_sub(deploy_skipped_stale);
        RequestKind::FetchLobby
            .run_with_timeout(state.fetch_after_deploy(instance_id, stale_not_founds))
            .await?
    } else {
        fetched.ok_or(Error::Disconnected)?
    };
    let node = state.node_status().await.unwrap_or_default();
    info!(
        target: "discovery",
        %key,
        rooms = lobby.len(),
        ring_peers = node.ring_peers,
        transport_peers = node.transport_peers,
        ring_location = ?node.ring_location,
        "lobby ready"
    );
    Ok((key, lobby))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;

    use freenet_stdlib::client_api::{
        ClientRequest, ContractRequest, ContractResponse, HostResponse, NetworkInfo,
        NodeDiagnosticsResponse, NodeInfo, QueryResponse,
    };
    use freenet_stdlib::prelude::{
        ContractCode, ContractKey, Parameters, WrappedContract, WrappedState,
    };
    use tokio_tungstenite::tungstenite::Message;

    use super::fetch_or_deploy_lobby;
    use crate::discovery;
    use discovery::Error;
    use discovery::freenet;
    use discovery::freenet::Client;
    use discovery::freenet::DeployPolicy;
    use discovery::freenet::FETCH_MAX_SENDS;
    use discovery::freenet::FETCH_RESEND_AFTER;
    use discovery::freenet::FETCH_SILENCE_LIMIT;

    fn contract_key() -> ContractKey {
        WrappedContract::new(
            Arc::new(ContractCode::from(freenet::LOBBY_CONTRACT_WASM.to_vec())),
            Parameters::from(Vec::new()),
        )
        .key
    }

    fn get_response(key: ContractKey, lobby: &discovery::Lobby) -> HostResponse {
        HostResponse::ContractResponse(ContractResponse::GetResponse {
            key,
            contract: None,
            state: WrappedState::new(bincode::serialize(lobby).unwrap_or_default()),
        })
    }

    fn not_found(key: ContractKey) -> HostResponse {
        HostResponse::ContractResponse(ContractResponse::NotFound {
            instance_id: *key.id(),
        })
    }

    fn put_response(key: ContractKey) -> HostResponse {
        HostResponse::ContractResponse(ContractResponse::PutResponse { key })
    }

    fn reachable_diagnostics() -> HostResponse {
        HostResponse::QueryResponse(QueryResponse::NodeDiagnostics(NodeDiagnosticsResponse {
            node_info: Some(NodeInfo {
                peer_id: "peer".to_string(),
                is_gateway: false,
                location: Some("0.5".to_string()),
                listening_address: None,
                uptime_seconds: 1,
            }),
            network_info: Some(NetworkInfo {
                connected_peers: Vec::new(),
                active_connections: 1,
            }),
            subscriptions: Vec::new(),
            contract_states: HashMap::new(),
            system_metrics: None,
            connected_peers_detailed: Vec::new(),
        }))
    }

    fn no_peers() -> HostResponse {
        HostResponse::QueryResponse(QueryResponse::ConnectedPeers { peers: Vec::new() })
    }

    fn is_lobby_get(message: &Message) -> bool {
        match message {
            Message::Binary(bytes) => matches!(
                bincode::deserialize::<ClientRequest<'_>>(bytes.as_ref()),
                Ok(ClientRequest::ContractOp(ContractRequest::Get { .. }))
            ),
            _ => false,
        }
    }

    fn is_lobby_put(message: &Message) -> bool {
        match message {
            Message::Binary(bytes) => matches!(
                bincode::deserialize::<ClientRequest<'_>>(bytes.as_ref()),
                Ok(ClientRequest::ContractOp(ContractRequest::Put { .. }))
            ),
            _ => false,
        }
    }

    #[tokio::test]
    async fn test_usage() {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let mut client = Client { write, read };
        let lobby = discovery::Lobby::default();
        assert!(
            fetch_or_deploy_lobby(
                &mut client,
                &[],
                &[],
                DeployPolicy::FetchOrDeploy,
                Duration::from_millis(1),
                &lobby,
            )
            .await
            .is_err()
        );
    }

    #[tokio::test(start_paused = true)]
    async fn resent_fetch_answers_second_send() {
        let key = contract_key();
        let lobby = discovery::Lobby::default();
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reachable_diagnostics())).ok();
        read_tx.send(Ok(no_peers())).ok();
        let mut client = Client { write, read };
        let fetch = fetch_or_deploy_lobby(
            &mut client,
            &[],
            &[],
            DeployPolicy::FetchOnly,
            Duration::from_secs(1),
            &lobby,
        );
        let driver = async {
            assert!(write_rx.recv().await.is_some());
            assert!(write_rx.recv().await.is_some());
            assert!(write_rx.recv().await.as_ref().is_some_and(is_lobby_get));
            tokio::time::advance(FETCH_RESEND_AFTER).await;
            assert!(write_rx.recv().await.as_ref().is_some_and(is_lobby_get));
            read_tx.send(Ok(get_response(key, &lobby))).ok();
            let mut resent_gets: usize = 0;
            while let Ok(message) = write_rx.try_recv() {
                if is_lobby_get(&message) {
                    resent_gets = resent_gets.saturating_add(1);
                }
            }
            resent_gets
        };
        let (outcome, resent_gets) = tokio::join!(fetch, driver);
        assert_eq!(outcome.ok().map(|(_, found)| found), Some(lobby));
        assert_eq!(resent_gets, 0);
    }

    #[tokio::test(start_paused = true)]
    async fn silent_fetch_sends_no_more_than_max() {
        let key = contract_key();
        let lobby = discovery::Lobby::default();
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reachable_diagnostics())).ok();
        read_tx.send(Ok(no_peers())).ok();
        let mut client = Client { write, read };
        let fetch = fetch_or_deploy_lobby(
            &mut client,
            &[],
            &[],
            DeployPolicy::FetchOnly,
            Duration::from_secs(1),
            &lobby,
        );
        let driver = async {
            assert!(write_rx.recv().await.is_some());
            assert!(write_rx.recv().await.is_some());
            let mut gets: usize = 0;
            if write_rx.recv().await.as_ref().is_some_and(is_lobby_get) {
                gets = gets.saturating_add(1);
            }
            while gets < FETCH_MAX_SENDS {
                tokio::time::advance(FETCH_RESEND_AFTER).await;
                if write_rx.recv().await.as_ref().is_some_and(is_lobby_get) {
                    gets = gets.saturating_add(1);
                }
            }
            tokio::time::advance(FETCH_RESEND_AFTER).await;
            assert!(write_rx.try_recv().is_err());
            tokio::time::advance(freenet::FETCH_TIMEOUT).await;
            read_tx.send(Ok(get_response(key, &lobby))).ok();
            while let Ok(message) = write_rx.try_recv() {
                if is_lobby_get(&message) {
                    gets = gets.saturating_add(1);
                }
            }
            gets
        };
        let (outcome, gets) = tokio::join!(fetch, driver);
        assert_eq!(outcome.ok().map(|(_, found)| found), Some(lobby));
        assert_eq!(gets, FETCH_MAX_SENDS);
    }

    #[tokio::test(start_paused = true)]
    async fn fast_fetch_sends_once() {
        let key = contract_key();
        let lobby = discovery::Lobby::default();
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reachable_diagnostics())).ok();
        read_tx.send(Ok(no_peers())).ok();
        read_tx.send(Ok(get_response(key, &lobby))).ok();
        let mut client = Client { write, read };
        let fetch = fetch_or_deploy_lobby(
            &mut client,
            &[],
            &[],
            DeployPolicy::FetchOnly,
            Duration::from_secs(1),
            &lobby,
        );
        let driver = async {
            assert!(write_rx.recv().await.is_some());
            assert!(write_rx.recv().await.is_some());
            let mut gets: usize = 0;
            if write_rx.recv().await.as_ref().is_some_and(is_lobby_get) {
                gets = gets.saturating_add(1);
            }
            while let Ok(message) = write_rx.try_recv() {
                if is_lobby_get(&message) {
                    gets = gets.saturating_add(1);
                }
            }
            gets
        };
        let (outcome, gets) = tokio::join!(fetch, driver);
        assert_eq!(outcome.ok().map(|(_, found)| found), Some(lobby));
        assert_eq!(gets, 1);
    }

    #[tokio::test(start_paused = true)]
    async fn deploy_skips_stale_get_replies() {
        let key = contract_key();
        let lobby = discovery::Lobby::default();
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reachable_diagnostics())).ok();
        read_tx.send(Ok(no_peers())).ok();
        read_tx.send(Ok(not_found(key))).ok();
        read_tx.send(Ok(not_found(key))).ok();
        read_tx.send(Ok(get_response(key, &lobby))).ok();
        read_tx.send(Ok(put_response(key))).ok();
        read_tx.send(Ok(get_response(key, &lobby))).ok();
        let mut client = Client { write, read };
        let fetch = fetch_or_deploy_lobby(
            &mut client,
            &[],
            &[],
            DeployPolicy::FetchOrDeploy,
            Duration::from_secs(1),
            &lobby,
        );
        let driver = async {
            let mut gets: usize = 0;
            let mut puts: usize = 0;
            for _ in 0..5 {
                let message = write_rx.recv().await;
                assert!(message.as_ref().is_some());
                if message.as_ref().is_some_and(is_lobby_get) {
                    gets = gets.saturating_add(1);
                }
                if message.as_ref().is_some_and(is_lobby_put) {
                    puts = puts.saturating_add(1);
                }
            }
            (gets, puts)
        };
        let (outcome, (gets, puts)) = tokio::join!(fetch, driver);
        assert_eq!(outcome.ok().map(|(_, found)| found), Some(lobby));
        assert_eq!(gets, 2);
        assert_eq!(puts, 1);
    }

    #[tokio::test(start_paused = true)]
    async fn stale_not_found_after_deploy_is_discarded() {
        let key = contract_key();
        let lobby = discovery::Lobby::default();
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reachable_diagnostics())).ok();
        read_tx.send(Ok(no_peers())).ok();
        let mut client = Client { write, read };
        let fetch = fetch_or_deploy_lobby(
            &mut client,
            &[],
            &[],
            DeployPolicy::FetchOrDeploy,
            Duration::from_secs(1),
            &lobby,
        );
        let driver = async {
            assert!(write_rx.recv().await.is_some());
            assert!(write_rx.recv().await.is_some());
            let mut gets: usize = 0;
            if write_rx.recv().await.as_ref().is_some_and(is_lobby_get) {
                gets = gets.saturating_add(1);
            }
            tokio::time::advance(FETCH_RESEND_AFTER).await;
            if write_rx.recv().await.as_ref().is_some_and(is_lobby_get) {
                gets = gets.saturating_add(1);
            }
            read_tx.send(Ok(not_found(key))).ok();
            read_tx.send(Ok(put_response(key))).ok();
            read_tx.send(Ok(not_found(key))).ok();
            read_tx.send(Ok(get_response(key, &lobby))).ok();
            let mut puts: usize = 0;
            if write_rx.recv().await.as_ref().is_some_and(is_lobby_put) {
                puts = puts.saturating_add(1);
            }
            if write_rx.recv().await.as_ref().is_some_and(is_lobby_get) {
                gets = gets.saturating_add(1);
            }
            while let Ok(message) = write_rx.try_recv() {
                if is_lobby_get(&message) {
                    gets = gets.saturating_add(1);
                }
                if is_lobby_put(&message) {
                    puts = puts.saturating_add(1);
                }
            }
            (gets, puts)
        };
        let (outcome, (gets, puts)) = tokio::join!(fetch, driver);
        assert_eq!(outcome.ok().map(|(_, found)| found), Some(lobby));
        assert_eq!(gets, 3);
        assert_eq!(puts, 1);
    }

    #[tokio::test(start_paused = true)]
    async fn genuine_not_found_after_deploy_still_fails() {
        let key = contract_key();
        let lobby = discovery::Lobby::default();
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reachable_diagnostics())).ok();
        read_tx.send(Ok(no_peers())).ok();
        read_tx.send(Ok(not_found(key))).ok();
        read_tx.send(Ok(put_response(key))).ok();
        read_tx.send(Ok(not_found(key))).ok();
        let mut client = Client { write, read };
        let fetch = fetch_or_deploy_lobby(
            &mut client,
            &[],
            &[],
            DeployPolicy::FetchOrDeploy,
            Duration::from_secs(1),
            &lobby,
        );
        let driver = async {
            let mut gets: usize = 0;
            let mut puts: usize = 0;
            for _ in 0..5 {
                let message = write_rx.recv().await;
                assert!(message.as_ref().is_some());
                if message.as_ref().is_some_and(is_lobby_get) {
                    gets = gets.saturating_add(1);
                }
                if message.as_ref().is_some_and(is_lobby_put) {
                    puts = puts.saturating_add(1);
                }
            }
            (gets, puts)
        };
        let (outcome, (gets, puts)) = tokio::join!(fetch, driver);
        assert!(matches!(outcome, Err(Error::ContractNotFound)));
        assert_eq!(gets, 2);
        assert_eq!(puts, 1);
    }

    #[tokio::test(start_paused = true)]
    async fn silent_fetch_or_deploy_deploys_at_silence_limit() {
        let key = contract_key();
        let lobby = discovery::Lobby::default();
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reachable_diagnostics())).ok();
        read_tx.send(Ok(no_peers())).ok();
        let mut client = Client { write, read };
        let fetch = fetch_or_deploy_lobby(
            &mut client,
            &[],
            &[],
            DeployPolicy::FetchOrDeploy,
            Duration::from_secs(1),
            &lobby,
        );
        let driver = async {
            assert!(write_rx.recv().await.is_some());
            assert!(write_rx.recv().await.is_some());
            assert!(write_rx.recv().await.as_ref().is_some_and(is_lobby_get));
            tokio::time::advance(FETCH_RESEND_AFTER).await;
            assert!(write_rx.recv().await.as_ref().is_some_and(is_lobby_get));
            assert!(write_rx.try_recv().is_err());
            tokio::time::advance(FETCH_SILENCE_LIMIT.checked_sub(FETCH_RESEND_AFTER).unwrap())
                .await;
            assert!(write_rx.recv().await.as_ref().is_some_and(is_lobby_put));
            read_tx.send(Ok(put_response(key))).ok();
            assert!(write_rx.recv().await.as_ref().is_some_and(is_lobby_get));
            read_tx.send(Ok(get_response(key, &lobby))).ok();
            (FETCH_MAX_SENDS, 1)
        };
        let (outcome, (gets, puts)) = tokio::join!(fetch, driver);
        assert_eq!(outcome.ok().map(|(_, found)| found), Some(lobby));
        assert_eq!(gets, FETCH_MAX_SENDS);
        assert_eq!(puts, 1);
    }

    #[tokio::test(start_paused = true)]
    async fn fetch_or_deploy_answer_before_silence_avoids_deploy() {
        let key = contract_key();
        let lobby = discovery::Lobby::default();
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reachable_diagnostics())).ok();
        read_tx.send(Ok(no_peers())).ok();
        let mut client = Client { write, read };
        let fetch = fetch_or_deploy_lobby(
            &mut client,
            &[],
            &[],
            DeployPolicy::FetchOrDeploy,
            Duration::from_secs(1),
            &lobby,
        );
        let driver = async {
            assert!(write_rx.recv().await.is_some());
            assert!(write_rx.recv().await.is_some());
            let mut gets: usize = 0;
            if write_rx.recv().await.as_ref().is_some_and(is_lobby_get) {
                gets = gets.saturating_add(1);
            }
            tokio::time::advance(FETCH_RESEND_AFTER).await;
            if write_rx.recv().await.as_ref().is_some_and(is_lobby_get) {
                gets = gets.saturating_add(1);
            }
            tokio::time::advance(
                FETCH_SILENCE_LIMIT
                    .checked_sub(FETCH_RESEND_AFTER)
                    .unwrap()
                    .checked_sub(Duration::from_secs(1))
                    .unwrap(),
            )
            .await;
            read_tx.send(Ok(get_response(key, &lobby))).ok();
            let mut puts: usize = 0;
            while let Ok(message) = write_rx.try_recv() {
                if is_lobby_put(&message) {
                    puts = puts.saturating_add(1);
                }
                if is_lobby_get(&message) {
                    gets = gets.saturating_add(1);
                }
            }
            (gets, puts)
        };
        let (outcome, (gets, puts)) = tokio::join!(fetch, driver);
        assert_eq!(outcome.ok().map(|(_, found)| found), Some(lobby));
        assert_eq!(gets, FETCH_MAX_SENDS);
        assert_eq!(puts, 0);
    }

    #[tokio::test(start_paused = true)]
    async fn fetch_only_ignores_silence_limit() {
        let key = contract_key();
        let lobby = discovery::Lobby::default();
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reachable_diagnostics())).ok();
        read_tx.send(Ok(no_peers())).ok();
        let mut client = Client { write, read };
        let fetch = fetch_or_deploy_lobby(
            &mut client,
            &[],
            &[],
            DeployPolicy::FetchOnly,
            Duration::from_secs(1),
            &lobby,
        );
        let driver = async {
            assert!(write_rx.recv().await.is_some());
            assert!(write_rx.recv().await.is_some());
            let mut gets: usize = 0;
            if write_rx.recv().await.as_ref().is_some_and(is_lobby_get) {
                gets = gets.saturating_add(1);
            }
            tokio::time::advance(FETCH_RESEND_AFTER).await;
            if write_rx.recv().await.as_ref().is_some_and(is_lobby_get) {
                gets = gets.saturating_add(1);
            }
            tokio::time::advance(FETCH_SILENCE_LIMIT.checked_sub(FETCH_RESEND_AFTER).unwrap())
                .await;
            let mut puts: usize = 0;
            while let Ok(message) = write_rx.try_recv() {
                if is_lobby_put(&message) {
                    puts = puts.saturating_add(1);
                }
                if is_lobby_get(&message) {
                    gets = gets.saturating_add(1);
                }
            }
            assert_eq!(puts, 0);
            tokio::time::advance(
                freenet::FETCH_TIMEOUT
                    .checked_sub(FETCH_SILENCE_LIMIT)
                    .unwrap(),
            )
            .await;
            while let Ok(message) = write_rx.try_recv() {
                if is_lobby_put(&message) {
                    puts = puts.saturating_add(1);
                }
                if is_lobby_get(&message) {
                    gets = gets.saturating_add(1);
                }
            }
            assert_eq!(puts, 0);
            read_tx.send(Ok(get_response(key, &lobby))).ok();
            while let Ok(message) = write_rx.try_recv() {
                if is_lobby_put(&message) {
                    puts = puts.saturating_add(1);
                }
                if is_lobby_get(&message) {
                    gets = gets.saturating_add(1);
                }
            }
            (gets, puts)
        };
        let (outcome, (gets, puts)) = tokio::join!(fetch, driver);
        assert_eq!(outcome.ok().map(|(_, found)| found), Some(lobby));
        assert_eq!(gets, FETCH_MAX_SENDS);
        assert_eq!(puts, 0);
    }

    #[tokio::test(start_paused = true)]
    async fn silent_deploy_discards_two_stale_not_founds() {
        let key = contract_key();
        let lobby = discovery::Lobby::default();
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reachable_diagnostics())).ok();
        read_tx.send(Ok(no_peers())).ok();
        let mut client = Client { write, read };
        let fetch = fetch_or_deploy_lobby(
            &mut client,
            &[],
            &[],
            DeployPolicy::FetchOrDeploy,
            Duration::from_secs(1),
            &lobby,
        );
        let driver = async {
            assert!(write_rx.recv().await.is_some());
            assert!(write_rx.recv().await.is_some());
            let mut gets: usize = 0;
            if write_rx.recv().await.as_ref().is_some_and(is_lobby_get) {
                gets = gets.saturating_add(1);
            }
            tokio::time::advance(FETCH_RESEND_AFTER).await;
            if write_rx.recv().await.as_ref().is_some_and(is_lobby_get) {
                gets = gets.saturating_add(1);
            }
            tokio::time::advance(FETCH_SILENCE_LIMIT.checked_sub(FETCH_RESEND_AFTER).unwrap())
                .await;
            let mut puts: usize = 0;
            if write_rx.recv().await.as_ref().is_some_and(is_lobby_put) {
                puts = puts.saturating_add(1);
            }
            read_tx.send(Ok(put_response(key))).ok();
            if write_rx.recv().await.as_ref().is_some_and(is_lobby_get) {
                gets = gets.saturating_add(1);
            }
            read_tx.send(Ok(not_found(key))).ok();
            read_tx.send(Ok(not_found(key))).ok();
            read_tx.send(Ok(get_response(key, &lobby))).ok();
            (gets, puts)
        };
        let (outcome, (gets, puts)) = tokio::join!(fetch, driver);
        assert_eq!(outcome.ok().map(|(_, found)| found), Some(lobby));
        assert_eq!(gets, FETCH_MAX_SENDS.saturating_add(1));
        assert_eq!(puts, 1);
    }

    #[tokio::test(start_paused = true)]
    async fn silent_deploy_third_not_found_fails() {
        let key = contract_key();
        let lobby = discovery::Lobby::default();
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reachable_diagnostics())).ok();
        read_tx.send(Ok(no_peers())).ok();
        let mut client = Client { write, read };
        let fetch = fetch_or_deploy_lobby(
            &mut client,
            &[],
            &[],
            DeployPolicy::FetchOrDeploy,
            Duration::from_secs(1),
            &lobby,
        );
        let driver = async {
            assert!(write_rx.recv().await.is_some());
            assert!(write_rx.recv().await.is_some());
            let mut gets: usize = 0;
            if write_rx.recv().await.as_ref().is_some_and(is_lobby_get) {
                gets = gets.saturating_add(1);
            }
            tokio::time::advance(FETCH_RESEND_AFTER).await;
            if write_rx.recv().await.as_ref().is_some_and(is_lobby_get) {
                gets = gets.saturating_add(1);
            }
            tokio::time::advance(FETCH_SILENCE_LIMIT.checked_sub(FETCH_RESEND_AFTER).unwrap())
                .await;
            let mut puts: usize = 0;
            if write_rx.recv().await.as_ref().is_some_and(is_lobby_put) {
                puts = puts.saturating_add(1);
            }
            read_tx.send(Ok(put_response(key))).ok();
            if write_rx.recv().await.as_ref().is_some_and(is_lobby_get) {
                gets = gets.saturating_add(1);
            }
            read_tx.send(Ok(not_found(key))).ok();
            read_tx.send(Ok(not_found(key))).ok();
            read_tx.send(Ok(not_found(key))).ok();
            (gets, puts)
        };
        let (outcome, (gets, puts)) = tokio::join!(fetch, driver);
        assert!(matches!(outcome, Err(Error::ContractNotFound)));
        assert_eq!(gets, FETCH_MAX_SENDS.saturating_add(1));
        assert_eq!(puts, 1);
    }
}
