mod direct_dial_peer;
mod lobby_peer;
mod lobby_room_peer;
mod meet_in_room;
mod mesh_convergence;
mod ui_peer;

pub use direct_dial_peer::DirectDialPeer;
pub use freenet_libp2p_bevy_plugin::lobby_room::testing::harness::{BrpNetwork, LobbySetup};
pub use freenet_libp2p_bevy_plugin::testing::Network;
pub use freenet_libp2p_bevy_plugin::testing::harness::{
    DayMillis, Deadlines, LogFile, PEER_WINDOWS, Peer, Polling,
};
pub use lobby_peer::LobbyPeer;
pub use lobby_room_peer::LobbyRoomPeer;
pub use meet_in_room::meet_in_room;
pub use mesh_convergence::expect_meshed_within_budget;
pub use ui_peer::UiPeer;

pub async fn poll_until<T, Probe, Done>(
    label: &str,
    deadline: std::time::Duration,
    probe: Probe,
    done: Done,
) -> Result<T, String>
where
    T: std::fmt::Debug,
    Probe: AsyncFnMut() -> Result<T, String>,
    Done: Fn(&T) -> bool,
{
    Polling::from(deadline).until(label, probe, done).await
}
