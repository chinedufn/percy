use clap::Parser;
use percy_cli::PercyCli;

fn main() -> anyhow::Result<()> {
    PercyCli::parse().run()
}
