mod brp_network;
mod direct_dial_peer;
mod expect_canvases_converge;
mod lobby_peer;
mod lobby_room_peer;
mod meet_in_room;
mod mesh_convergence;
mod starboard_peer;
mod stroke;
mod ui_peer;
mod wait_until_any_history_written;

pub use brp_network::BrpNetwork;
pub use direct_dial_peer::DirectDialPeer;
pub use expect_canvases_converge::expect_canvases_converge;
pub use freenet_libp2p_bevy_plugin::testing::Network;
pub use freenet_libp2p_bevy_plugin::testing::harness::{
    DayMillis, Deadlines, LogFile, PEER_WINDOWS, Peer, Polling, UnixNanos,
};
pub use lobby_peer::LobbyPeer;
pub use lobby_room_peer::LobbyRoomPeer;
pub use meet_in_room::meet_in_room;
pub use mesh_convergence::expect_meshed_within_budget;
pub use starboard_peer::StarboardPeer;
pub use stroke::Stroke;
pub use ui_peer::UiPeer;
pub use wait_until_any_history_written::wait_until_any_history_written;

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
