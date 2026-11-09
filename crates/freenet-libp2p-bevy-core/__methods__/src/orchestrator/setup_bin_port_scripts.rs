use std::path::PathBuf;

use crate::testing;

pub async fn setup_bin_port_scripts(
    orchestrator: &testing::Orchestrator,
    room: &str,
) -> Result<(PathBuf, Option<u16>, Vec<String>), String> {
    let bin = orchestrator.build_game_binary()?;
    let orchestrator_port = if matches!(orchestrator.network, testing::Network::Fake) {
        let server = testing::FakeNetworkServer::start().await?;
        Some(*server)
    } else {
        None
    };
    let mut scripts: Vec<String> = Vec::new();
    for index in 0..orchestrator.peers.len() {
        scripts.push(orchestrator.peer_script(index, room)?);
    }
    Ok((bin, orchestrator_port, scripts))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let _ = stringify!(super::setup_bin_port_scripts);
    }
}
