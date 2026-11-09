use std::sync::Arc;
use std::time::Duration;

use freenet_stdlib::prelude::*;
use tracing::{info, warn};

use crate::freenet;
use freenet::Client;
use freenet::DeployPolicy;
use freenet::Error;
use freenet::FETCH_MAX_SENDS;
use freenet::FETCH_RESEND_AFTER;
use freenet::FETCH_SILENCE_LIMIT;
use freenet::RequestKind;

#[rustfmt::skip]
pub async fn fetch_or_deploy<S>(
    state: &mut Client,
    contract_wasm: &[u8],
    params: &[u8],
    deploy: DeployPolicy,
    gateway_wait: Duration,
    seed_state: &S,
) -> Result<(ContractKey, S), Error>
where
    S: serde::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    state.wait_until_node_reachable(gateway_wait).await?;
    let code = Arc::new(ContractCode::from(contract_wasm.to_vec()));
    let wrapped = WrappedContract::new(code, Parameters::from(params.to_vec()));
    let key = wrapped.key;
    let instance_id = *key.id();
    info!(target: "freenet", %key, ?deploy, "contract fetch start");
    let started = std::time::Instant::now();
    let mut sends: usize = 0;
    let fetch_all = async {
        loop {
            state.send_lobby_fetch(instance_id)?;
            sends = sends.saturating_add(1);
            if sends < FETCH_MAX_SENDS {
                match tokio::time::timeout(FETCH_RESEND_AFTER, state.await_outstanding_fetch::<S>())
                    .await
                {
                    Err(_) => {
                        info!(
                            target: "freenet",
                            sends,
                            elapsed_ms = started.elapsed().as_millis(),
                            "contract fetch resent"
                        );
                    }
                    Ok(settled) => break settled,
                }
            } else {
                break state.await_outstanding_fetch::<S>().await;
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
        warn!(target: "freenet", sends, elapsed_ms, "contract fetch silent, deploying");
        warn!(target: "freenet", elapsed_ms, sends, error = "Silent", "contract fetch failed");
        (Some(sends), None)
    } else {
        let first = match raw_first {
            Err(Error::Timeout(RequestKind::FetchLobby)) => {
                info!(target: "freenet", %key, "contract fetch timed out, awaiting the node's answer");
                state.await_outstanding_fetch::<S>().await
            }
            first => first,
        };
        let elapsed_ms = started.elapsed().as_millis();
        match first {
            Ok(state_found) => {
                info!(target: "freenet", elapsed_ms, sends, "contract fetch found");
                (None, Some(state_found))
            }
            Err(e) => {
                warn!(target: "freenet", elapsed_ms, sends, error = ?e, "contract fetch failed");
                if is_fetch_or_deploy && matches!(e, Error::ContractNotFound) {
                    (Some(sends.saturating_sub(1)), None)
                } else {
                    return Err(e);
                }
            }
        }
    };
    let found = if let Some(unanswered_sends) = deploy_unanswered {
        info!(target: "freenet", %key, "contract deploy start");
        let container = ContractContainer::from(ContractWasmAPIVersion::V1(wrapped));
        let deploy_skipped_stale = RequestKind::DeployLobby
            .run_with_timeout(state.deploy_contract(container, seed_state))
            .await
            .inspect_err(|e| warn!(target: "freenet", error = ?e, "contract deploy failed"))?;
        let stale_not_founds = unanswered_sends.saturating_sub(deploy_skipped_stale);
        RequestKind::FetchLobby
            .run_with_timeout(state.fetch_after_deploy::<S>(instance_id, stale_not_founds))
            .await?
    } else {
        fetched.ok_or(Error::Disconnected)?
    };
    let node = state.node_status().await.unwrap_or_default();
    info!(
        target: "freenet",
        %key,
        ring_peers = node.ring_peers,
        transport_peers = node.transport_peers,
        ring_location = ?node.ring_location,
        "contract ready"
    );
    Ok((key, found))
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

    use serde::{Deserialize, Serialize};

    use super::fetch_or_deploy;
    use crate::freenet;
    use freenet::Client;
    use freenet::DeployPolicy;
    use freenet::Error;
    use freenet::FETCH_MAX_SENDS;
    use freenet::FETCH_RESEND_AFTER;
    use freenet::FETCH_SILENCE_LIMIT;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
    struct TestState {
        value: u64,
    }

    fn contract_key() -> ContractKey {
        WrappedContract::new(
            Arc::new(ContractCode::from(vec![0; 4])),
            Parameters::from(Vec::new()),
        )
        .key
    }

    fn get_response(key: ContractKey, lobby: &TestState) -> HostResponse {
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

    fn is_contract_get(message: &Message) -> bool {
        match message {
            Message::Binary(bytes) => matches!(
                bincode::deserialize::<ClientRequest<'_>>(bytes.as_ref()),
                Ok(ClientRequest::ContractOp(ContractRequest::Get { .. }))
            ),
            _ => false,
        }
    }

    fn is_contract_put(message: &Message) -> bool {
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
        let lobby = TestState::default();
        assert!(
            fetch_or_deploy(
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
        let lobby = TestState::default();
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reachable_diagnostics())).ok();
        read_tx.send(Ok(no_peers())).ok();
        let mut client = Client { write, read };
        let fetch = fetch_or_deploy(
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
            assert!(write_rx.recv().await.as_ref().is_some_and(is_contract_get));
            tokio::time::advance(FETCH_RESEND_AFTER).await;
            assert!(write_rx.recv().await.as_ref().is_some_and(is_contract_get));
            read_tx.send(Ok(get_response(key, &lobby))).ok();
            let mut resent_gets: usize = 0;
            while let Ok(message) = write_rx.try_recv() {
                if is_contract_get(&message) {
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
        let lobby = TestState::default();
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reachable_diagnostics())).ok();
        read_tx.send(Ok(no_peers())).ok();
        let mut client = Client { write, read };
        let fetch = fetch_or_deploy(
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
            if write_rx.recv().await.as_ref().is_some_and(is_contract_get) {
                gets = gets.saturating_add(1);
            }
            while gets < FETCH_MAX_SENDS {
                tokio::time::advance(FETCH_RESEND_AFTER).await;
                if write_rx.recv().await.as_ref().is_some_and(is_contract_get) {
                    gets = gets.saturating_add(1);
                }
            }
            tokio::time::advance(FETCH_RESEND_AFTER).await;
            assert!(write_rx.try_recv().is_err());
            tokio::time::advance(freenet::FETCH_TIMEOUT).await;
            read_tx.send(Ok(get_response(key, &lobby))).ok();
            while let Ok(message) = write_rx.try_recv() {
                if is_contract_get(&message) {
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
        let lobby = TestState::default();
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reachable_diagnostics())).ok();
        read_tx.send(Ok(no_peers())).ok();
        read_tx.send(Ok(get_response(key, &lobby))).ok();
        let mut client = Client { write, read };
        let fetch = fetch_or_deploy(
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
            if write_rx.recv().await.as_ref().is_some_and(is_contract_get) {
                gets = gets.saturating_add(1);
            }
            while let Ok(message) = write_rx.try_recv() {
                if is_contract_get(&message) {
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
        let lobby = TestState::default();
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
        let fetch = fetch_or_deploy(
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
                if message.as_ref().is_some_and(is_contract_get) {
                    gets = gets.saturating_add(1);
                }
                if message.as_ref().is_some_and(is_contract_put) {
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
        let lobby = TestState::default();
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reachable_diagnostics())).ok();
        read_tx.send(Ok(no_peers())).ok();
        let mut client = Client { write, read };
        let fetch = fetch_or_deploy(
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
            if write_rx.recv().await.as_ref().is_some_and(is_contract_get) {
                gets = gets.saturating_add(1);
            }
            tokio::time::advance(FETCH_RESEND_AFTER).await;
            if write_rx.recv().await.as_ref().is_some_and(is_contract_get) {
                gets = gets.saturating_add(1);
            }
            read_tx.send(Ok(not_found(key))).ok();
            read_tx.send(Ok(put_response(key))).ok();
            read_tx.send(Ok(not_found(key))).ok();
            read_tx.send(Ok(get_response(key, &lobby))).ok();
            let mut puts: usize = 0;
            if write_rx.recv().await.as_ref().is_some_and(is_contract_put) {
                puts = puts.saturating_add(1);
            }
            if write_rx.recv().await.as_ref().is_some_and(is_contract_get) {
                gets = gets.saturating_add(1);
            }
            while let Ok(message) = write_rx.try_recv() {
                if is_contract_get(&message) {
                    gets = gets.saturating_add(1);
                }
                if is_contract_put(&message) {
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
        let lobby = TestState::default();
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reachable_diagnostics())).ok();
        read_tx.send(Ok(no_peers())).ok();
        read_tx.send(Ok(not_found(key))).ok();
        read_tx.send(Ok(put_response(key))).ok();
        read_tx.send(Ok(not_found(key))).ok();
        let mut client = Client { write, read };
        let fetch = fetch_or_deploy(
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
                if message.as_ref().is_some_and(is_contract_get) {
                    gets = gets.saturating_add(1);
                }
                if message.as_ref().is_some_and(is_contract_put) {
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
        let lobby = TestState::default();
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reachable_diagnostics())).ok();
        read_tx.send(Ok(no_peers())).ok();
        let mut client = Client { write, read };
        let fetch = fetch_or_deploy(
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
            assert!(write_rx.recv().await.as_ref().is_some_and(is_contract_get));
            tokio::time::advance(FETCH_RESEND_AFTER).await;
            assert!(write_rx.recv().await.as_ref().is_some_and(is_contract_get));
            assert!(write_rx.try_recv().is_err());
            tokio::time::advance(FETCH_SILENCE_LIMIT.checked_sub(FETCH_RESEND_AFTER).unwrap())
                .await;
            assert!(write_rx.recv().await.as_ref().is_some_and(is_contract_put));
            read_tx.send(Ok(put_response(key))).ok();
            assert!(write_rx.recv().await.as_ref().is_some_and(is_contract_get));
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
        let lobby = TestState::default();
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reachable_diagnostics())).ok();
        read_tx.send(Ok(no_peers())).ok();
        let mut client = Client { write, read };
        let fetch = fetch_or_deploy(
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
            if write_rx.recv().await.as_ref().is_some_and(is_contract_get) {
                gets = gets.saturating_add(1);
            }
            tokio::time::advance(FETCH_RESEND_AFTER).await;
            if write_rx.recv().await.as_ref().is_some_and(is_contract_get) {
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
                if is_contract_put(&message) {
                    puts = puts.saturating_add(1);
                }
                if is_contract_get(&message) {
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
        let lobby = TestState::default();
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reachable_diagnostics())).ok();
        read_tx.send(Ok(no_peers())).ok();
        let mut client = Client { write, read };
        let fetch = fetch_or_deploy(
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
            if write_rx.recv().await.as_ref().is_some_and(is_contract_get) {
                gets = gets.saturating_add(1);
            }
            tokio::time::advance(FETCH_RESEND_AFTER).await;
            if write_rx.recv().await.as_ref().is_some_and(is_contract_get) {
                gets = gets.saturating_add(1);
            }
            tokio::time::advance(FETCH_SILENCE_LIMIT.checked_sub(FETCH_RESEND_AFTER).unwrap())
                .await;
            let mut puts: usize = 0;
            while let Ok(message) = write_rx.try_recv() {
                if is_contract_put(&message) {
                    puts = puts.saturating_add(1);
                }
                if is_contract_get(&message) {
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
                if is_contract_put(&message) {
                    puts = puts.saturating_add(1);
                }
                if is_contract_get(&message) {
                    gets = gets.saturating_add(1);
                }
            }
            assert_eq!(puts, 0);
            read_tx.send(Ok(get_response(key, &lobby))).ok();
            while let Ok(message) = write_rx.try_recv() {
                if is_contract_put(&message) {
                    puts = puts.saturating_add(1);
                }
                if is_contract_get(&message) {
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
        let lobby = TestState::default();
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reachable_diagnostics())).ok();
        read_tx.send(Ok(no_peers())).ok();
        let mut client = Client { write, read };
        let fetch = fetch_or_deploy(
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
            if write_rx.recv().await.as_ref().is_some_and(is_contract_get) {
                gets = gets.saturating_add(1);
            }
            tokio::time::advance(FETCH_RESEND_AFTER).await;
            if write_rx.recv().await.as_ref().is_some_and(is_contract_get) {
                gets = gets.saturating_add(1);
            }
            tokio::time::advance(FETCH_SILENCE_LIMIT.checked_sub(FETCH_RESEND_AFTER).unwrap())
                .await;
            let mut puts: usize = 0;
            if write_rx.recv().await.as_ref().is_some_and(is_contract_put) {
                puts = puts.saturating_add(1);
            }
            read_tx.send(Ok(put_response(key))).ok();
            if write_rx.recv().await.as_ref().is_some_and(is_contract_get) {
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
        let lobby = TestState::default();
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reachable_diagnostics())).ok();
        read_tx.send(Ok(no_peers())).ok();
        let mut client = Client { write, read };
        let fetch = fetch_or_deploy(
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
            if write_rx.recv().await.as_ref().is_some_and(is_contract_get) {
                gets = gets.saturating_add(1);
            }
            tokio::time::advance(FETCH_RESEND_AFTER).await;
            if write_rx.recv().await.as_ref().is_some_and(is_contract_get) {
                gets = gets.saturating_add(1);
            }
            tokio::time::advance(FETCH_SILENCE_LIMIT.checked_sub(FETCH_RESEND_AFTER).unwrap())
                .await;
            let mut puts: usize = 0;
            if write_rx.recv().await.as_ref().is_some_and(is_contract_put) {
                puts = puts.saturating_add(1);
            }
            read_tx.send(Ok(put_response(key))).ok();
            if write_rx.recv().await.as_ref().is_some_and(is_contract_get) {
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
