//! The Percy CLI

#![deny(missing_docs)]

use crate::subcommand::PercySubcommand;
use clap::Parser;

mod run;
mod subcommand;

/// The Percy CLI.
#[derive(Debug, Parser)]
pub struct PercyCli {
    #[clap(subcommand)]
    subcommand: PercySubcommand,
}
