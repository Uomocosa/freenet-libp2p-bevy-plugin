use bevy::prelude::*;
use clap::Parser;

use input_fidelity_e2e::args::ProbeArgs;

fn main() -> AppExit {
    let args = ProbeArgs::parse();
    ProbeArgs::init_tracing();
    args.run()
}
