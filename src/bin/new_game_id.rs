use freenet_libp2p_bevy_plugin::identity;

fn main() {
    println!("{}", identity::GameId::new_game_id());
}

#[cfg(test)]
mod tests {
    use freenet_libp2p_bevy_plugin::identity;

    #[test]
    fn test_usage() {
        let token = identity::GameId::new_game_id();
        assert_eq!(token.len(), 16);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
