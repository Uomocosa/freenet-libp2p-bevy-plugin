use crate::testing;

pub fn peer_args_with_token(
    net: &testing::harness::BrpNetwork,
    username: &str,
    window: (i32, i32),
    token: &str,
) -> Vec<String> {
    let (left, top) = window;
    let node_dir = net.run_dir.join(format!("{username}-freenet"));
    let mut args = vec![
        "--username".to_string(),
        username.to_string(),
        "--token".to_string(),
        token.to_string(),
        "--network".to_string(),
        net.network_arg().to_string(),
        "--window-position".to_string(),
        format!("{left},{top}"),
        "--freenet-node-dir".to_string(),
        node_dir.display().to_string(),
    ];
    if let Some(server) = &net.orchestrator {
        args.push("--orchestrator-port".to_string());
        args.push((**server).to_string());
    }
    if matches!(net.network, testing::Network::DirectDial) {
        args.push("--identity-file".to_string());
        args.push(
            net.run_dir
                .join(format!("{username}.identity"))
                .display()
                .to_string(),
        );
        args.push("--peer-book-file".to_string());
        args.push(
            net.run_dir
                .join(format!("{username}.peers.json"))
                .display()
                .to_string(),
        );
    }
    args
}

#[cfg(test)]
mod tests {
    use super::peer_args_with_token;
    use crate::testing;

    #[test]
    fn test_usage() {
        let net = testing::brp_network(testing::Network::DirectDial, "brp-1", "/tmp/brp-args");
        let args = peer_args_with_token(&net, "ada", (3, 4), "brp-9");
        assert!(args.contains(&"--token".to_string()));
        assert!(args.contains(&"brp-9".to_string()));
        assert!(args.contains(&"3,4".to_string()));
        assert!(args.contains(&"--identity-file".to_string()));
        assert!(!args.iter().any(|arg| arg == "--orchestrator-port"));
    }
}
