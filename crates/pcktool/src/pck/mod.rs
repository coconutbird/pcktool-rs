//! PCK (Wwise package) file format support.
//!
//! A PCK file contains:
//! - A language string map
//! - Sound bank entries (`.bnk` files)
//! - Streaming file entries (`.wem` files played via streaming)
//! - External file entries (64-bit ID, additional loose media)
//!
//! The on-disk layout is:
//! ```text
//! [AKPK tag: u32][header_size: u32][version: u32]
//! [lang_map_size: u32][banks_lut_size: u32][stm_lut_size: u32][ext_lut_size: u32]
//! [language string map]
//! [soundbank LUT]
//! [streaming file LUT]
//! [external file LUT]
//! [file data (block-aligned)]
//! ```

mod entry;
mod string_map;
mod writer;

pub use entry::{ExternalFileEntry, FileEntry, SoundBankEntry, StreamingFileEntry};
pub use string_map::StringMap;
pub use writer::PckWriter;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use hashbrown::HashMap;

use crate::error::{Error, Result};
use crate::hash::fourcc;
use zerocopy::{FromBytes, IntoBytes, little_endian as le};

/// AKPK magic tag.
const AKPK_TAG: u32 = fourcc(b"AKPK");
/// Expected PCK version.
const PCK_VERSION: u32 = 0x1;

/// On-disk PCK header (fixed portion after the tag).
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes)]
#[repr(C, packed)]
pub struct PckHeader {
    pub header_size: le::U32,
    pub version: le::U32,
    pub language_map_size: le::U32,
    pub sound_banks_lut_size: le::U32,
    pub streaming_files_lut_size: le::U32,
    pub external_files_lut_size: le::U32,
}

/// On-disk LUT entry for 32-bit keyed files (soundbanks, streaming).
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes)]
#[repr(C, packed)]
pub struct LutEntry32 {
    pub id: le::U32,
    pub block_size: le::U32,
    pub file_size: le::I32,
    pub start_block: le::U32,
    pub language_id: le::U32,
}

/// On-disk LUT entry for 64-bit keyed files (external).
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes)]
#[repr(C, packed)]
pub struct LutEntry64 {
    pub id: le::U64,
    pub block_size: le::U32,
    pub file_size: le::I32,
    pub start_block: le::U32,
    pub language_id: le::U32,
}

/// A parsed PCK file. Borrows file data from the underlying byte slice.
#[derive(Debug)]
pub struct PckFile<'a> {
    /// Language ID → name mapping.
    pub languages: HashMap<u32, String>,
    /// Sound bank entries (borrow data from the source slice).
    pub sound_banks: Vec<SoundBankEntry<'a>>,
    /// Streaming file entries (WEM files).
    pub streaming_files: Vec<StreamingFileEntry<'a>>,
    /// External file entries (64-bit IDs).
    pub external_files: Vec<ExternalFileEntry<'a>>,
}

impl<'a> PckFile<'a> {
    /// Parse a PCK file from a byte slice. Entry data borrows from `data` (zero-copy).
    pub fn parse(data: &'a [u8]) -> Result<Self> {
        if data.len() < 4 {
            return Err(Error::UnexpectedEof {
                offset: 0,
                needed: 4,
                available: data.len() as u64,
            });
        }

        let tag = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if tag != AKPK_TAG {
            return Err(Error::InvalidMagic {
                expected: AKPK_TAG,
                actual: tag,
            });
        }

        let header_offset = 4usize;
        let header_size = core::mem::size_of::<PckHeader>();
        if data.len() < header_offset + header_size {
            return Err(Error::UnexpectedEof {
                offset: header_offset as u64,
                needed: header_size as u64,
                available: (data.len() - header_offset) as u64,
            });
        }

        let header = PckHeader::read_from_bytes(&data[header_offset..header_offset + header_size])
            .map_err(|e| Error::ZeroCopyCast {
                offset: header_offset as u64,
                reason: format!("{e:?}"),
            })?;

        let version = header.version.get();
        if version != PCK_VERSION {
            return Err(Error::UnsupportedVersion(version));
        }

        let lang_map_size = header.language_map_size.get() as usize;
        let banks_lut_size = header.sound_banks_lut_size.get() as usize;
        let stm_lut_size = header.streaming_files_lut_size.get() as usize;
        let ext_lut_size = header.external_files_lut_size.get() as usize;

        let mut cursor = header_offset + header_size;

        // Parse language map
        let lang_end = cursor + lang_map_size;
        if lang_end > data.len() {
            return Err(Error::UnexpectedEof {
                offset: cursor as u64,
                needed: lang_map_size as u64,
                available: (data.len() - cursor) as u64,
            });
        }
        let languages = StringMap::parse(&data[cursor..lang_end])?;
        cursor = lang_end;

        // Parse soundbank LUT (32-bit keys)
        let banks_end = cursor + banks_lut_size;
        let sound_banks = Self::parse_lut_32(&data[cursor..banks_end], data)?;
        cursor = banks_end;

        // Parse streaming file LUT (32-bit keys)
        let stm_end = cursor + stm_lut_size;
        let streaming_files = Self::parse_lut_32(&data[cursor..stm_end], data)?;
        cursor = stm_end;

        // Parse external file LUT (64-bit keys)
        let ext_end = cursor + ext_lut_size;
        let external_files = Self::parse_lut_64(&data[cursor..ext_end], data)?;
        let _ = ext_end;

        Ok(PckFile {
            languages: languages.entries,
            sound_banks,
            streaming_files,
            external_files,
        })
    }

    /// Look up a language name by ID, returning "SFX" for ID 0 or "Unknown" if missing.
    pub fn language_name(&self, id: u32) -> &str {
        if id == 0 {
            return "SFX";
        }
        self.languages
            .get(&id)
            .map(|s| s.as_str())
            .unwrap_or("Unknown")
    }

    /// Find a WEM by source ID — checks streaming files first, then embedded media in soundbanks.
    pub fn find_wem(&self, source_id: u32) -> Option<&'a [u8]> {
        // Check streaming files
        for entry in &self.streaming_files {
            if entry.id == source_id {
                return Some(entry.data);
            }
        }
        // Check embedded media in soundbanks
        for bank_entry in &self.sound_banks {
            if let Ok(bank) = crate::bnk::SoundBank::parse(bank_entry.data)
                && let Some(media) = bank.find_media(source_id)
            {
                // media borrows from bank_entry.data which borrows from 'a
                // but bank is a local — we need to return data from the original slice
                // The media slice is a sub-slice of bank_entry.data, so this is safe
                let offset = media.as_ptr() as usize - bank_entry.data.as_ptr() as usize;
                return Some(&bank_entry.data[offset..offset + media.len()]);
            }
        }
        None
    }

    // ── Internal LUT parsers ──

    fn parse_lut_32<E: FileEntry<'a, u32>>(lut_data: &[u8], file_data: &'a [u8]) -> Result<Vec<E>> {
        if lut_data.len() < 4 {
            return Ok(Vec::new());
        }
        let count =
            u32::from_le_bytes([lut_data[0], lut_data[1], lut_data[2], lut_data[3]]) as usize;
        let entry_size = core::mem::size_of::<LutEntry32>();
        let mut entries = Vec::with_capacity(count);
        let mut offset = 4usize;

        for i in 0..count {
            if offset + entry_size > lut_data.len() {
                return Err(Error::InvalidEntry {
                    index: i as u32,
                    reason: format!("LUT truncated at entry {i}"),
                });
            }
            let raw = LutEntry32::read_from_bytes(&lut_data[offset..offset + entry_size]).map_err(
                |e| Error::ZeroCopyCast {
                    offset: offset as u64,
                    reason: format!("{e:?}"),
                },
            )?;

            let file_size = raw.file_size.get();
            if file_size < 0 {
                return Err(Error::InvalidEntry {
                    index: i as u32,
                    reason: format!("negative file size {file_size}"),
                });
            }
            let file_size = file_size as usize;
            let actual_offset = raw.start_block.get() as u64 * raw.block_size.get() as u64;
            let start = actual_offset as usize;
            let end = start + file_size;
            if end > file_data.len() {
                return Err(Error::InvalidEntry {
                    index: i as u32,
                    reason: format!(
                        "data range 0x{start:X}..0x{end:X} exceeds file size 0x{:X}",
                        file_data.len()
                    ),
                });
            }

            entries.push(E::from_raw_32(
                raw.id.get(),
                raw.block_size.get(),
                raw.start_block.get(),
                raw.language_id.get(),
                &file_data[start..end],
            ));
            offset += entry_size;
        }
        Ok(entries)
    }

    fn parse_lut_64(lut_data: &[u8], file_data: &'a [u8]) -> Result<Vec<ExternalFileEntry<'a>>> {
        if lut_data.len() < 4 {
            return Ok(Vec::new());
        }
        let count =
            u32::from_le_bytes([lut_data[0], lut_data[1], lut_data[2], lut_data[3]]) as usize;
        let entry_size = core::mem::size_of::<LutEntry64>();
        let mut entries = Vec::with_capacity(count);
        let mut offset = 4usize;

        for i in 0..count {
            if offset + entry_size > lut_data.len() {
                return Err(Error::InvalidEntry {
                    index: i as u32,
                    reason: format!("LUT truncated at entry {i}"),
                });
            }
            let raw = LutEntry64::read_from_bytes(&lut_data[offset..offset + entry_size]).map_err(
                |e| Error::ZeroCopyCast {
                    offset: offset as u64,
                    reason: format!("{e:?}"),
                },
            )?;

            let file_size = raw.file_size.get();
            if file_size < 0 {
                return Err(Error::InvalidEntry {
                    index: i as u32,
                    reason: format!("negative file size {file_size}"),
                });
            }
            let file_size = file_size as usize;
            let actual_offset = raw.start_block.get() as u64 * raw.block_size.get() as u64;
            let start = actual_offset as usize;
            let end = start + file_size;
            if end > file_data.len() {
                return Err(Error::InvalidEntry {
                    index: i as u32,
                    reason: format!(
                        "data range 0x{start:X}..0x{end:X} exceeds file size 0x{:X}",
                        file_data.len()
                    ),
                });
            }

            entries.push(ExternalFileEntry {
                id: raw.id.get(),
                block_size: raw.block_size.get(),
                start_block: raw.start_block.get(),
                language_id: raw.language_id.get(),
                data: &file_data[start..end],
            });
            offset += entry_size;
        }
        Ok(entries)
    }
}
