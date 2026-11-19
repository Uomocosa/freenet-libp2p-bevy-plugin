use clap::Parser;

use crate::testing;

#[derive(Debug, Parser)]
pub struct NetworkArgsProbe {
    #[command(flatten)]
    pub network: testing::NetworkArgs,
}
