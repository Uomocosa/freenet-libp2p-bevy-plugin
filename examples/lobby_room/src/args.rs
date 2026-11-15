use std::path::PathBuf;

use clap::Parser;
use freenet_libp2p_bevy_plugin::p2p::TransportMode;
use lobby_room::app::UiChoice;

#[derive(Debug, Parser)]
#[command(name = "lobby_room")]
pub struct Args {
    #[arg(long)]
    pub username: String,
    #[arg(long)]
    pub token: Option<String>,
    #[arg(long, value_enum, default_value = "both")]
    pub transport: TransportMode,
    /// Which lobby UI to run: the plugin's default one or the example's replacement.
    #[arg(long, value_enum, default_value = "default")]
    pub ui: UiChoice,
    #[arg(long = "room-ping")]
    pub is_room_ping_enabled: bool,
    /// Folder the embedded Freenet node keeps its config, data and logs in (default: a temp dir).
    #[arg(long)]
    pub freenet_node_dir: Option<PathBuf>,
    /// File the friends drawer saves its peer book to (default: memory only).
    #[arg(long)]
    pub peer_book_file: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use freenet_libp2p_bevy_plugin::p2p::TransportMode;
    use lobby_room::app::UiChoice;

    #[test]
    fn transport_and_ui_parse_to_real_enums() {
        let args = super::Args::try_parse_from([
            "lobby_room",
            "--username",
            "p1",
            "--transport",
            "quic",
            "--ui",
            "custom",
        ])
        .expect("args");
        assert_eq!(args.transport, TransportMode::Quic);
        assert_eq!(args.ui, UiChoice::Custom);
    }

    #[test]
    fn peer_book_file_is_optional_and_parses() {
        use std::path::PathBuf;
        let args = super::Args::try_parse_from(["lobby_room", "--username", "p1"]).expect("args");
        assert_eq!(args.peer_book_file, None);
        let args = super::Args::try_parse_from([
            "lobby_room",
            "--username",
            "p1",
            "--peer-book-file",
            "/tmp/p1.friends.json",
        ])
        .expect("args");
        assert_eq!(
            args.peer_book_file,
            Some(PathBuf::from("/tmp/p1.friends.json"))
        );
    }
}
