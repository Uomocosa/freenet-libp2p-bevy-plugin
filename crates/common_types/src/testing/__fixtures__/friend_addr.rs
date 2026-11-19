pub fn friend_addr() -> String {
    let id = "12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC";
    format!("/ip4/127.0.0.1/tcp/4001/p2p/{id}")
}

#[cfg(test)]
mod tests {
    use super::friend_addr;

    #[test]
    fn test_usage() {
        let addr = friend_addr();
        assert!(addr.starts_with("/ip4/127.0.0.1/tcp/4001/p2p/"));
        assert!(addr.ends_with("12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC"));
    }
}
