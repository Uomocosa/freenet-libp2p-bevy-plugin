#![allow(dead_code)]
#![allow(unused_imports)]

mod build_game;
pub use build_game::build_game;

mod cleanup_stale;
pub use cleanup_stale::cleanup_stale;

mod finish_record;
pub use finish_record::finish_record;

mod log_parse;
pub use log_parse::{
    CanvasTick, Tick, canvas_ticks, history_write_bytes, last_canvas, last_tick, log_contains,
    log_matches, marker_times, read_head, senders_seen_in_room,
};

mod poke;
pub use poke::poke;

mod require_x11;
pub use require_x11::require_x11;

mod scenario;
pub use scenario::{Report, Scenario, run_scenario};

mod shell_escape;
pub use shell_escape::shell_escape;

mod spawn_app;
pub use spawn_app::spawn_app;

mod speed_clip;
pub use speed_clip::speed_clip;

mod start_record;
pub use start_record::start_record;

mod terminal_guard;
pub use terminal_guard::TerminalGuard;

mod tile;
pub use tile::tile;

mod rings_ready;
pub use rings_ready::rings_ready;

mod wakeup_screen;
pub use wakeup_screen::wakeup_screen;
