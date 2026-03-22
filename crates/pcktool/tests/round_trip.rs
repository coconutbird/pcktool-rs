//! Round-trip integration test: read a PCK file, write it back, and verify
//! that the re-parsed output matches the original.
//!
//! Requires `Sounds.pck` at the workspace root. The test is ignored if the
//! file is not present so CI can still pass without the fixture.

use pcktool::pck::{PckFile, WriteEntry, Writer};
use std::fs;
use std::path::Path;

/// Path to the test fixture relative to the workspace root.
const PCK_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../Sounds.pck");

#[test]
fn round_trip_sounds_pck() {
    let path = Path::new(PCK_PATH);
    if !path.exists() {
        eprintln!(
            "skipping round_trip test: Sounds.pck not found at {}",
            path.display()
        );
        return;
    }

    let original_bytes = fs::read(path).expect("failed to read Sounds.pck");
    let original = PckFile::parse(&original_bytes).expect("failed to parse original PCK");

    // ── Build a Writer from the parsed data ──
    let mut writer = Writer::new();

    // Languages
    for (&id, name) in &original.languages {
        writer.languages.insert(id, name.clone());
    }

    // Sound banks
    for entry in &original.sound_banks {
        writer.sound_banks.push(WriteEntry {
            id: entry.id,
            block_size: entry.block_size,
            language_id: entry.language_id,
            data: entry.data.to_vec(),
        });
    }

    // Streaming files
    for entry in &original.streaming_files {
        writer.streaming_files.push(WriteEntry {
            id: entry.id,
            block_size: entry.block_size,
            language_id: entry.language_id,
            data: entry.data.to_vec(),
        });
    }

    // External files
    for entry in &original.external_files {
        writer.external_files.push(WriteEntry {
            id: entry.id,
            block_size: entry.block_size,
            language_id: entry.language_id,
            data: entry.data.to_vec(),
        });
    }

    // ── Serialize ──
    let written_bytes = writer.to_bytes();

    // ── Re-parse the written bytes ──
    let reparsed = PckFile::parse(&written_bytes).expect("failed to parse round-tripped PCK");

    // ── Compare ──
    // Languages
    assert_eq!(
        original.languages.len(),
        reparsed.languages.len(),
        "language count mismatch"
    );
    for (&id, name) in &original.languages {
        assert_eq!(
            reparsed.languages.get(&id).map(|s| s.as_str()),
            Some(name.as_str()),
            "language {id} mismatch"
        );
    }

    // Sound banks
    assert_eq!(
        original.sound_banks.len(),
        reparsed.sound_banks.len(),
        "sound bank count mismatch"
    );
    for (i, (orig, re)) in original
        .sound_banks
        .iter()
        .zip(reparsed.sound_banks.iter())
        .enumerate()
    {
        assert_eq!(orig.id, re.id, "bank[{i}] id mismatch");
        assert_eq!(
            orig.language_id, re.language_id,
            "bank[{i}] language mismatch"
        );
        assert_eq!(
            orig.data.len(),
            re.data.len(),
            "bank[{i}] data length mismatch"
        );
        assert_eq!(orig.data, re.data, "bank[{i}] data mismatch");
    }

    // Streaming files
    assert_eq!(
        original.streaming_files.len(),
        reparsed.streaming_files.len(),
        "streaming file count mismatch"
    );
    for (i, (orig, re)) in original
        .streaming_files
        .iter()
        .zip(reparsed.streaming_files.iter())
        .enumerate()
    {
        assert_eq!(orig.id, re.id, "stm[{i}] id mismatch");
        assert_eq!(
            orig.language_id, re.language_id,
            "stm[{i}] language mismatch"
        );
        assert_eq!(
            orig.data.len(),
            re.data.len(),
            "stm[{i}] data length mismatch"
        );
        assert_eq!(orig.data, re.data, "stm[{i}] data mismatch");
    }

    // External files
    assert_eq!(
        original.external_files.len(),
        reparsed.external_files.len(),
        "external file count mismatch"
    );
    for (i, (orig, re)) in original
        .external_files
        .iter()
        .zip(reparsed.external_files.iter())
        .enumerate()
    {
        assert_eq!(orig.id, re.id, "ext[{i}] id mismatch");
        assert_eq!(
            orig.language_id, re.language_id,
            "ext[{i}] language mismatch"
        );
        assert_eq!(orig.data, re.data, "ext[{i}] data mismatch");
    }

    eprintln!(
        "round-trip OK: {} languages, {} banks, {} streaming, {} external",
        reparsed.languages.len(),
        reparsed.sound_banks.len(),
        reparsed.streaming_files.len(),
        reparsed.external_files.len(),
    );
}
