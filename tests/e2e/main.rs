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

#[path = "../../examples/shared/e2e_report/__basic__/enums.rs"]
mod check_layer;
#[path = "../../examples/shared/e2e_report/expectation_result.rs"]
mod expectation_result;
#[path = "../../examples/shared/e2e_report/__basic__/structs.rs"]
mod freenet_layer_result;
#[path = "../../examples/shared/e2e_report/__basic__/resources.rs"]
mod peer_run_result;
#[path = "../../examples/shared/e2e_report/scenario_expectation.rs"]
#[allow(dead_code)]
mod scenario_expectation;

#[path = "../starboard_steps/mod.rs"]
mod starboard_steps;

mod direct_dial;
mod friends_reconnect;
mod harness;
mod known_peers;
mod starboard_bad_token;
mod starboard_e2e;
mod starboard_history_holder_killed;
mod starboard_history_restore;
mod starboard_history_room_gone;
mod starboard_late_join;
mod starboard_leave_rejoin;
mod starboard_peer_killed;
mod starboard_simultaneous_paint;
