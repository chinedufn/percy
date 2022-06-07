use clap::Parser;

use self::preview::Preview;

mod preview;

#[derive(Debug, Parser)]
pub(crate) enum PercySubcommand {
    /// Commands related to previewing a Percy application's components.
    Preview(Preview),
}
