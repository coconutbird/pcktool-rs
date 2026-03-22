use clap::Args;
use pcktool::pck::PckFile;

use super::mmap_file;

#[derive(Args)]
pub struct ListArgs {
    /// Path to a .pck file.
    pub file: std::path::PathBuf,

    /// Show streaming files instead of sound banks.
    #[arg(short = 's', long)]
    pub streaming: bool,

    /// Show external files instead of sound banks.
    #[arg(short = 'e', long)]
    pub external: bool,
}

pub fn run(args: ListArgs) -> anyhow::Result<()> {
    let mmap = mmap_file(&args.file)?;
    let pck = PckFile::parse(&mmap)?;

    if args.streaming {
        println!("{:<12} {:<10} {:<10} Block", "ID", "Size", "Language");
        println!("{}", "─".repeat(50));
        for entry in &pck.streaming_files {
            println!(
                "0x{:08X}  {:<10} {:<10} {}",
                entry.id,
                entry.data.len(),
                pck.language_name(entry.language_id),
                entry.start_block,
            );
        }
        println!("\nTotal: {} streaming files", pck.streaming_files.len());
    } else if args.external {
        println!("{:<18} {:<10} {:<10} Block", "ID", "Size", "Language");
        println!("{}", "─".repeat(56));
        for entry in &pck.external_files {
            println!(
                "0x{:016X}  {:<10} {:<10} {}",
                entry.id,
                entry.data.len(),
                pck.language_name(entry.language_id),
                entry.start_block,
            );
        }
        println!("\nTotal: {} external files", pck.external_files.len());
    } else {
        println!("{:<12} {:<10} {:<10} Block", "ID", "Size", "Language");
        println!("{}", "─".repeat(50));
        for entry in &pck.sound_banks {
            println!(
                "0x{:08X}  {:<10} {:<10} {}",
                entry.id,
                entry.data.len(),
                pck.language_name(entry.language_id),
                entry.start_block,
            );
        }
        println!("\nTotal: {} sound banks", pck.sound_banks.len());
    }

    Ok(())
}
