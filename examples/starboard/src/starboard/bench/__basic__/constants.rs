use std::time::Duration;

pub const BENCH_PEER_COUNTS: [usize; 2] = [2, 8];
pub const BENCH_OPS_PER_SECOND: u32 = 60;
pub const BENCH_SECONDS: u32 = 20;
pub const BENCH_WARMUP_SECONDS: u32 = 2;
pub const BENCH_FRAME: Duration = Duration::from_nanos(16_666_667);
pub const BENCH_CONVERGENCE_BUDGET: Duration = Duration::from_secs(30);
pub const BENCH_GAME: &str = "starboard-bench";
pub const BENCH_ROOM: &str = "bench-room";
pub const BENCH_SIZE_INDEX: u8 = 0;
pub const BENCH_TICKS: u32 =
    BENCH_OPS_PER_SECOND.saturating_mul(BENCH_SECONDS.saturating_add(BENCH_WARMUP_SECONDS));
pub const STROKE_BAND_WIDTH: usize = 85;
pub const STROKE_START_OFFSET: usize = 5;
pub const STROKE_LENGTH: usize = 35;
pub const REPAINT_TILE_COUNTS: [usize; 3] = [1, 32, 1024];
pub const REPAINT_ROUNDS: u32 = 200;
