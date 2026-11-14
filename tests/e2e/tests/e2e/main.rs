#[allow(dead_code)]
#[path = "../common/build_game.rs"]
mod build_game;
#[path = "../common/cleanup_stale.rs"]
mod cleanup_stale;
#[path = "../common/finish_record.rs"]
mod finish_record;
#[path = "../common/require_x11.rs"]
mod require_x11;
#[path = "../common/speed_clip.rs"]
mod speed_clip;
#[path = "../common/start_record.rs"]
mod start_record;
#[path = "../common/tile.rs"]
mod tile;
#[path = "../common/wakeup_screen.rs"]
mod wakeup_screen;

use examples_shared::e2e_report::check_layer;
use examples_shared::e2e_report::expectation_result;
use examples_shared::e2e_report::peer_run_result;
use examples_shared::e2e_report::scenario_expectation;

use e2e_steps::starboard_steps;

mod brp;
mod brp_concurrent_strokes_converge;
mod brp_lobby_room_two_peers_meet;
mod brp_one_peer_paints;
mod direct_dial;
mod friends_reconnect;
mod github_checkpoint_board;
mod harness;
mod known_peers;
mod local_checkpoint_board;
mod starboard_bad_token;
mod starboard_e2e;
mod starboard_history_holder_killed;
mod starboard_history_restore;
mod starboard_history_room_gone;
mod starboard_late_join;
mod starboard_leave_rejoin;
mod starboard_peer_killed;
mod starboard_simultaneous_paint;
