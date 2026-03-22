mod commands;

use clap::Parser;

#[derive(Parser)]
#[command(name = "pcktool", version, about = "Wwise PCK/BNK audio package tool")]
struct Cli {
    #[command(subcommand)]
    command: commands::Command,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    commands::run(cli.command)
}
