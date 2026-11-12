use std::path::PathBuf;

use crate::starboard;

impl starboard::canvas::Canvas {
    pub(crate) fn filled() -> Self {
        let mut canvas = Self::default();
        let cell = starboard::canvas::Layer {
            rank: 1,
            author: 1,
            color: Some(starboard::identity::PaletteColor(1)),
            mask: starboard::canvas::FULL_MASK,
        };
        let tile_side = u16::try_from(starboard::canvas::TILE_SIDE).unwrap_or(1);
        for index in 0..starboard::canvas::TILE_COUNT {
            let x = index % starboard::canvas::TILES_PER_SIDE;
            let y = index / starboard::canvas::TILES_PER_SIDE;
            let _ = starboard::canvas::write_cell(
                &mut canvas,
                starboard::canvas::CellPos {
                    x: u16::try_from(x).unwrap_or(0).saturating_mul(tile_side),
                    y: u16::try_from(y).unwrap_or(0).saturating_mul(tile_side),
                },
                cell,
            );
        }
        canvas
    }

    pub(crate) fn measure_repaint_costs() -> Vec<starboard::bench::RepaintCost> {
        let mut canvas = Self::filled();
        starboard::bench::REPAINT_TILE_COUNTS
            .iter()
            .map(|count| {
                starboard::bench::repaint_cost(
                    &mut canvas,
                    *count,
                    starboard::bench::REPAINT_ROUNDS,
                )
            })
            .collect()
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "benchmark: 2 and 8 in-process peers, 20 s each, writes examples/starboard/BENCH.md"]
async fn bench_starboard() {
    let mut reports = Vec::new();
    for peers in starboard::bench::BENCH_PEER_COUNTS {
        let report = starboard::bench::run_scenario(starboard::bench::ScenarioConfig {
            peers,
            ops_per_second: starboard::bench::BENCH_OPS_PER_SECOND,
            seconds: starboard::bench::BENCH_SECONDS,
        })
        .await;
        println!(
            "starboard bench peers={} achieved={:.1}/s p50={:?} p99={:?} received={:.1} KiB/s dropped={:.1}% convergence={:?} samples={}",
            report.config.peers,
            report.achieved_ops_per_second,
            report.latency.p50,
            report.latency.p99,
            report.received_bytes_per_second / 1024.0,
            report.dropped_percent,
            report.convergence,
            report.latency.samples,
        );
        reports.push(report);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    let repaint = starboard::canvas::Canvas::measure_repaint_costs();
    let markdown = starboard::bench::bench_markdown(
        &reports,
        &starboard::bench::MachineInfo::detect(),
        &starboard::bench::bench_date(),
        &repaint,
    );
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("BENCH.md");
    std::fs::write(&path, markdown).expect("BENCH.md written");
    println!("starboard bench wrote {}", path.display());
}
