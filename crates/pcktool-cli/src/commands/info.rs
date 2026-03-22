use clap::Args;
use pcktool::bnk::SoundBank;
use pcktool::pck::PckFile;

use super::mmap_file;

#[derive(Args)]
pub struct InfoArgs {
    /// Path to a .pck or .bnk file.
    pub file: std::path::PathBuf,
}

pub fn run(args: InfoArgs) -> anyhow::Result<()> {
    let ext = args
        .file
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let mmap = mmap_file(&args.file)?;

    match ext.as_str() {
        "pck" => info_pck(&mmap),
        "bnk" => info_bnk(&mmap),
        _ => anyhow::bail!("unsupported file extension '.{ext}' (expected .pck or .bnk)"),
    }
}

fn info_pck(data: &[u8]) -> anyhow::Result<()> {
    let pck = PckFile::parse(data)?;

    println!("PCK File Info");
    println!("─────────────────────────────────");
    println!("Languages:        {}", pck.languages.len());
    for (id, name) in &pck.languages {
        println!("  [{id}] {name}");
    }
    println!("Sound Banks:      {}", pck.sound_banks.len());
    println!("Streaming Files:  {}", pck.streaming_files.len());
    println!("External Files:   {}", pck.external_files.len());

    let total_bank_size: usize = pck.sound_banks.iter().map(|e| e.data.len()).sum();
    let total_stm_size: usize = pck.streaming_files.iter().map(|e| e.data.len()).sum();
    let total_ext_size: usize = pck.external_files.iter().map(|e| e.data.len()).sum();

    println!("─────────────────────────────────");
    println!("Bank data:        {} bytes", total_bank_size);
    println!("Streaming data:   {} bytes", total_stm_size);
    println!("External data:    {} bytes", total_ext_size);

    Ok(())
}

fn info_bnk(data: &[u8]) -> anyhow::Result<()> {
    let bank = SoundBank::parse(data)?;

    println!("BNK File Info");
    println!("─────────────────────────────────");
    println!("Version:          0x{:X}", bank.header.version);
    println!("Bank ID:          0x{:08X}", bank.header.id);
    println!("Language ID:      {}", bank.header.language_id);
    println!("Project ID:       {}", bank.header.project_id);
    println!("Feedback:         {}", bank.header.has_feedback());
    println!("Embedded Media:   {}", bank.media.len());

    let total_media: usize = bank.media.values().map(|d| d.len()).sum();
    println!("Media data:       {} bytes", total_media);

    if bank.hirc_data.is_some() {
        println!("HIRC:             present");
    }

    Ok(())
}
