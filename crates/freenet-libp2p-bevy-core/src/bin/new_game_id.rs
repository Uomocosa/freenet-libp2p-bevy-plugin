use freenet_libp2p_bevy_core::identity;

fn main() {
    println!("{}", identity::GameId::generate_new_random_game_id());
}

#[cfg(test)]
mod tests {
    use freenet_libp2p_bevy_core::identity;

    #[test]
    fn test_usage() {
        let token = identity::GameId::generate_new_random_game_id();
        assert_eq!(token.len(), 16);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
