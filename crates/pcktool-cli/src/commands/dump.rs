use std::fs;
use std::path::PathBuf;

use clap::Args;
use pcktool::bnk::SoundBank;
use pcktool::pck::PckFile;

use super::mmap_file;

#[derive(Args)]
pub struct DumpArgs {
    /// Path to a .pck or .bnk file.
    pub file: PathBuf,

    /// Output directory (default: <filename>_extracted).
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Only extract a specific source ID (hex or decimal).
    #[arg(long, value_parser = super::parse_id)]
    pub id: Option<u32>,
}

pub fn run(args: DumpArgs) -> anyhow::Result<()> {
    let ext = args
        .file
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let stem = args
        .file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let out_dir = args
        .output
        .unwrap_or_else(|| PathBuf::from(format!("{stem}_extracted")));

    fs::create_dir_all(&out_dir)?;

    let mmap = mmap_file(&args.file)?;

    match ext.as_str() {
        "pck" => dump_pck(&mmap, &out_dir, args.id),
        "bnk" => dump_bnk(&mmap, &out_dir, args.id),
        _ => anyhow::bail!("unsupported file extension '.{ext}'"),
    }
}

fn dump_pck(data: &[u8], out_dir: &std::path::Path, filter_id: Option<u32>) -> anyhow::Result<()> {
    let pck = PckFile::parse(data)?;
    let mut count = 0u32;

    // Dump sound banks
    let banks_dir = out_dir.join("banks");
    fs::create_dir_all(&banks_dir)?;
    for entry in &pck.sound_banks {
        if filter_id.is_some() && filter_id != Some(entry.id) {
            continue;
        }
        let path = banks_dir.join(format!("{:08X}.bnk", entry.id));
        fs::write(&path, entry.data)?;
        println!("  → {}", path.display());
        count += 1;

        // Also extract embedded media from each bank
        if let Ok(bank) = SoundBank::parse(entry.data) {
            if !bank.media.is_empty() {
                let media_dir = banks_dir.join(format!("{:08X}_media", entry.id));
                fs::create_dir_all(&media_dir)?;
                for (&id, &wem_data) in &bank.media {
                    let wem_path = media_dir.join(format!("{id:08X}.wem"));
                    fs::write(&wem_path, wem_data)?;
                    println!("    → {}", wem_path.display());
                    count += 1;
                }
            }
        }
    }

    // Dump streaming files
    let stm_dir = out_dir.join("streaming");
    fs::create_dir_all(&stm_dir)?;
    for entry in &pck.streaming_files {
        if filter_id.is_some() && filter_id != Some(entry.id) {
            continue;
        }
        let path = stm_dir.join(format!("{:08X}.wem", entry.id));
        fs::write(&path, entry.data)?;
        println!("  → {}", path.display());
        count += 1;
    }

    // Dump external files
    if !pck.external_files.is_empty() {
        let ext_dir = out_dir.join("external");
        fs::create_dir_all(&ext_dir)?;
        for entry in &pck.external_files {
            let path = ext_dir.join(format!("{:016X}.wem", entry.id));
            fs::write(&path, entry.data)?;
            println!("  → {}", path.display());
            count += 1;
        }
    }

    println!("\nExtracted {count} files to {}", out_dir.display());
    Ok(())
}

fn dump_bnk(data: &[u8], out_dir: &std::path::Path, filter_id: Option<u32>) -> anyhow::Result<()> {
    let bank = SoundBank::parse(data)?;
    let mut count = 0u32;

    for (&id, &wem_data) in &bank.media {
        if filter_id.is_some() && filter_id != Some(id) {
            continue;
        }
        let path = out_dir.join(format!("{id:08X}.wem"));
        fs::write(&path, wem_data)?;
        println!("  → {}", path.display());
        count += 1;
    }

    println!("\nExtracted {count} WEM files to {}", out_dir.display());
    Ok(())
}

