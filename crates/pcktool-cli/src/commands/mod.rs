mod dump;
mod info;
mod list;

use clap::Subcommand;

/// Parse a u32 from decimal or hex (0x prefix).
pub fn parse_id(s: &str) -> Result<u32, String> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).map_err(|e| format!("invalid hex ID '{s}': {e}"))
    } else {
        s.parse::<u32>().map_err(|e| format!("invalid ID '{s}': {e}"))
    }
}

/// Load a file via mmap and return the mapping.
pub fn mmap_file(path: &std::path::Path) -> anyhow::Result<memmap2::Mmap> {
    let file = std::fs::File::open(path)?;
    // SAFETY: we don't modify the file while the mmap is alive
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    Ok(mmap)
}

#[derive(Subcommand)]
pub enum Command {
    /// Show information about a PCK or BNK file.
    Info(info::InfoArgs),
    /// List all sound banks in a PCK file.
    #[command(alias = "ls")]
    List(list::ListArgs),
    /// Extract sound banks and WEM files from a PCK file.
    #[command(alias = "extract")]
    Dump(dump::DumpArgs),
}

pub fn run(cmd: Command) -> anyhow::Result<()> {
    match cmd {
        Command::Info(args) => info::run(args),
        Command::List(args) => list::run(args),
        Command::Dump(args) => dump::run(args),
    }
}

