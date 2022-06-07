use crate::{PercyCli, PercySubcommand};

impl PercyCli {
    /// Run the CLI.
    pub fn run(self) -> Result<(), anyhow::Error> {
        match self.subcommand {
            PercySubcommand::Preview(cmd) => cmd.run(),
        }
    }
}
