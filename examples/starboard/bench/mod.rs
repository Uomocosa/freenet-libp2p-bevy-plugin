#[path = "__basic__/mod.rs"]
pub mod basic;

mod bench_date;
mod bench_markdown;
mod bench_starboard;
mod machine_info;
mod percentiles;
mod repaint_cost;
mod run_scenario;
mod scripted_stroke;

pub use basic::constants::{
    BENCH_CONVERGENCE_BUDGET, BENCH_FRAME, BENCH_GAME, BENCH_OPS_PER_SECOND, BENCH_PEER_COUNTS,
    BENCH_ROOM, BENCH_SECONDS, BENCH_SIZE_INDEX, BENCH_TICKS, BENCH_WARMUP_SECONDS, REPAINT_ROUNDS,
    REPAINT_TILE_COUNTS, STROKE_BAND_WIDTH, STROKE_LENGTH, STROKE_START_OFFSET,
};
pub use basic::structs::{
    BenchReport, LatencyPercentiles, MachineInfo, PeerTraffic, RepaintCost, ScenarioConfig,
};
pub use bench_date::bench_date;
pub use bench_markdown::bench_markdown;
pub use machine_info::machine_info;
pub use percentiles::percentiles;
pub use repaint_cost::repaint_cost;
pub use run_scenario::run_scenario;
pub use scripted_stroke::scripted_stroke;
