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
pub use writer::{WriteEntry, Writer};

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use hashbrown::HashMap;

use crate::error::{Error, Result};
use crate::hash::fourcc;

/// AKPK magic tag.
const AKPK_TAG: u32 = fourcc(b"AKPK");
/// Expected PCK version.
const PCK_VERSION: u32 = 0x1;
/// Big-endian representation of version 1 (for detection).
const PCK_VERSION_BE: u32 = PCK_VERSION.swap_bytes();

/// Byte order of a PCK file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    /// Little-endian (standard on PC/Xbox).
    Little,
    /// Big-endian (e.g. `Sounds_china.pck`).
    Big,
}

impl ByteOrder {
    /// Read a `u16` from a 2-byte slice in this byte order.
    #[inline]
    pub fn read_u16(self, bytes: [u8; 2]) -> u16 {
        match self {
            Self::Little => u16::from_le_bytes(bytes),
            Self::Big => u16::from_be_bytes(bytes),
        }
    }

    /// Read a `u32` from a 4-byte slice in this byte order.
    #[inline]
    pub fn read_u32(self, bytes: [u8; 4]) -> u32 {
        match self {
            Self::Little => u32::from_le_bytes(bytes),
            Self::Big => u32::from_be_bytes(bytes),
        }
    }

    /// Read an `i32` from a 4-byte slice in this byte order.
    #[inline]
    pub fn read_i32(self, bytes: [u8; 4]) -> i32 {
        match self {
            Self::Little => i32::from_le_bytes(bytes),
            Self::Big => i32::from_be_bytes(bytes),
        }
    }

    /// Read a `u64` from an 8-byte slice in this byte order.
    #[inline]
    pub fn read_u64(self, bytes: [u8; 8]) -> u64 {
        match self {
            Self::Little => u64::from_le_bytes(bytes),
            Self::Big => u64::from_be_bytes(bytes),
        }
    }

    /// Write a `u16` in this byte order.
    #[inline]
    pub fn write_u16(self, v: u16) -> [u8; 2] {
        match self {
            Self::Little => v.to_le_bytes(),
            Self::Big => v.to_be_bytes(),
        }
    }

    /// Write a `u32` in this byte order.
    #[inline]
    pub fn write_u32(self, v: u32) -> [u8; 4] {
        match self {
            Self::Little => v.to_le_bytes(),
            Self::Big => v.to_be_bytes(),
        }
    }

    /// Write an `i32` in this byte order.
    #[inline]
    pub fn write_i32(self, v: i32) -> [u8; 4] {
        match self {
            Self::Little => v.to_le_bytes(),
            Self::Big => v.to_be_bytes(),
        }
    }

    /// Write a `u64` in this byte order.
    #[inline]
    pub fn write_u64(self, v: u64) -> [u8; 8] {
        match self {
            Self::Little => v.to_le_bytes(),
            Self::Big => v.to_be_bytes(),
        }
    }
}

/// Helper: read a `u32` from `data[offset..offset+4]` in the given byte order.
#[inline]
fn read_u32_at(data: &[u8], offset: usize, bo: ByteOrder) -> u32 {
    bo.read_u32([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

/// Helper: read an `i32` from `data[offset..offset+4]` in the given byte order.
#[inline]
fn read_i32_at(data: &[u8], offset: usize, bo: ByteOrder) -> i32 {
    bo.read_i32([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

/// Helper: read a `u64` from `data[offset..offset+8]` in the given byte order.
#[inline]
fn read_u64_at(data: &[u8], offset: usize, bo: ByteOrder) -> u64 {
    bo.read_u64([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ])
}

/// Standard header: header_size + version + lang_sz + banks_sz + stm_sz + ext_sz = 6 fields.
const HEADER_FIELDS_STANDARD: usize = 6;
/// Compact header (BE variant): header_size + version + lang_sz + banks_sz + stm_sz = 5 fields.
const HEADER_FIELDS_COMPACT: usize = 5;

/// Standard 32-bit LUT entry: id(4) + block_size(4) + file_size(4) + start_block(4) + language_id(4).
const LUT_ENTRY_32_STANDARD: usize = 20;
/// Extended 32-bit LUT entry (BE variant): id(4) + block_size(4) + pad(4) + file_size(4) + start_block(4) + language_id(4).
const LUT_ENTRY_32_EXTENDED: usize = 24;

/// Size of a 64-bit LUT entry on disk: id(8) + block_size(4) + file_size(4) + start_block(4) + language_id(4).
const LUT_ENTRY_64_SIZE: usize = 24;

/// A parsed PCK file. Borrows file data from the underlying byte slice.
#[derive(Debug)]
pub struct PckFile<'a> {
    /// Detected byte order of the source file.
    pub byte_order: ByteOrder,
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
    ///
    /// Endianness is auto-detected from the version field: if the raw bytes
    /// at offset 8 equal `0x00000001` the file is little-endian; if they equal
    /// `0x01000000` it is big-endian.
    pub fn parse(data: &'a [u8]) -> Result<Self> {
        // Need at least tag(4) + header_size(4) + version(4) = 12 bytes to detect endianness
        if data.len() < 12 {
            return Err(Error::UnexpectedEof {
                offset: 0,
                needed: 12,
                available: data.len() as u64,
            });
        }

        // Tag is ASCII "AKPK" — endian-neutral
        let tag = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if tag != AKPK_TAG {
            return Err(Error::InvalidMagic {
                expected: AKPK_TAG,
                actual: tag,
            });
        }

        // Detect endianness from the raw version field at offset 8
        let version_raw = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let bo = if version_raw == PCK_VERSION {
            ByteOrder::Little
        } else if version_raw == PCK_VERSION_BE {
            ByteOrder::Big
        } else {
            return Err(Error::UnsupportedVersion(version_raw));
        };

        // We need at least 5 header fields to start parsing.
        let header_offset = 4usize;
        let min_header = HEADER_FIELDS_COMPACT * 4;
        if data.len() < header_offset + min_header {
            return Err(Error::UnexpectedEof {
                offset: header_offset as u64,
                needed: min_header as u64,
                available: (data.len() - header_offset) as u64,
            });
        }

        let header_size_field = read_u32_at(data, header_offset, bo) as usize;
        // version already validated above
        let lang_map_size = read_u32_at(data, header_offset + 8, bo) as usize;
        let banks_lut_size = read_u32_at(data, header_offset + 12, bo) as usize;
        let stm_lut_size = read_u32_at(data, header_offset + 16, bo) as usize;

        // Detect whether the header has 5 fields (compact) or 6 (standard).
        // The compact form (seen in BE `Sounds_china.pck`) omits `ext_lut_size`.
        // header_size_field = meta_fields_size + sections_total.
        let compact_meta = (HEADER_FIELDS_COMPACT - 1) * 4; // version + 3 section sizes
        let compact_sum = compact_meta + lang_map_size + banks_lut_size + stm_lut_size;

        let (ext_lut_size, compact) = if compact_sum == header_size_field {
            // Compact header — no external files section
            (0usize, true)
        } else {
            // Standard header — read ext_lut_size
            if data.len() < header_offset + HEADER_FIELDS_STANDARD * 4 {
                return Err(Error::UnexpectedEof {
                    offset: header_offset as u64,
                    needed: (HEADER_FIELDS_STANDARD * 4) as u64,
                    available: (data.len() - header_offset) as u64,
                });
            }
            let ext_lut_size = read_u32_at(data, header_offset + 20, bo) as usize;
            (ext_lut_size, false)
        };

        let header_fields = if compact {
            HEADER_FIELDS_COMPACT
        } else {
            HEADER_FIELDS_STANDARD
        };
        // Compact headers use extended 24-byte LUT entries; standard use 20-byte.
        let lut_stride = if compact {
            LUT_ENTRY_32_EXTENDED
        } else {
            LUT_ENTRY_32_STANDARD
        };

        let mut cursor = header_offset + header_fields * 4;

        // Parse language map
        let lang_end = cursor + lang_map_size;
        if lang_end > data.len() {
            return Err(Error::UnexpectedEof {
                offset: cursor as u64,
                needed: lang_map_size as u64,
                available: (data.len() - cursor) as u64,
            });
        }
        let languages = StringMap::parse(&data[cursor..lang_end], bo)?;
        cursor = lang_end;

        // Parse soundbank LUT (32-bit keys)
        let banks_end = cursor + banks_lut_size;
        let sound_banks = Self::parse_lut_32(&data[cursor..banks_end], data, bo, lut_stride)?;
        cursor = banks_end;

        // Parse streaming file LUT (32-bit keys)
        let stm_end = cursor + stm_lut_size;
        let streaming_files = Self::parse_lut_32(&data[cursor..stm_end], data, bo, lut_stride)?;
        cursor = stm_end;

        // Parse external file LUT (64-bit keys)
        let ext_end = cursor + ext_lut_size;
        let external_files = Self::parse_lut_64(&data[cursor..ext_end], data, bo)?;
        let _ = ext_end;

        Ok(PckFile {
            byte_order: bo,
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

    fn parse_lut_32<E: FileEntry<'a, u32>>(
        lut_data: &[u8],
        file_data: &'a [u8],
        bo: ByteOrder,
        stride: usize,
    ) -> Result<Vec<E>> {
        if lut_data.len() < 4 {
            return Ok(Vec::new());
        }
        let count = read_u32_at(lut_data, 0, bo) as usize;
        if count == 0 {
            return Ok(Vec::new());
        }

        // Offset of file_size field within an entry (skips the extra padding in extended).
        let fs_off: usize = if stride == LUT_ENTRY_32_EXTENDED {
            12
        } else {
            8
        };

        let mut entries = Vec::with_capacity(count);
        let mut offset = 4usize;

        for i in 0..count {
            if offset + stride > lut_data.len() {
                return Err(Error::InvalidEntry {
                    index: i as u32,
                    reason: format!("LUT truncated at entry {i}"),
                });
            }

            let id = read_u32_at(lut_data, offset, bo);
            let block_size = read_u32_at(lut_data, offset + 4, bo);
            let file_size = read_i32_at(lut_data, offset + fs_off, bo);
            let start_block = read_u32_at(lut_data, offset + fs_off + 4, bo);
            let language_id = read_u32_at(lut_data, offset + fs_off + 8, bo);

            if file_size < 0 {
                return Err(Error::InvalidEntry {
                    index: i as u32,
                    reason: format!("negative file size {file_size}"),
                });
            }
            let file_size = file_size as usize;
            let actual_offset = start_block as u64 * block_size as u64;
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
                id,
                block_size,
                start_block,
                language_id,
                &file_data[start..end],
            ));
            offset += stride;
        }
        Ok(entries)
    }

    fn parse_lut_64(
        lut_data: &[u8],
        file_data: &'a [u8],
        bo: ByteOrder,
    ) -> Result<Vec<ExternalFileEntry<'a>>> {
        if lut_data.len() < 4 {
            return Ok(Vec::new());
        }
        let count = read_u32_at(lut_data, 0, bo) as usize;
        let mut entries = Vec::with_capacity(count);
        let mut offset = 4usize;

        for i in 0..count {
            if offset + LUT_ENTRY_64_SIZE > lut_data.len() {
                return Err(Error::InvalidEntry {
                    index: i as u32,
                    reason: format!("LUT truncated at entry {i}"),
                });
            }

            let id = read_u64_at(lut_data, offset, bo);
            let block_size = read_u32_at(lut_data, offset + 8, bo);
            let file_size = read_i32_at(lut_data, offset + 12, bo);
            let start_block = read_u32_at(lut_data, offset + 16, bo);
            let language_id = read_u32_at(lut_data, offset + 20, bo);

            if file_size < 0 {
                return Err(Error::InvalidEntry {
                    index: i as u32,
                    reason: format!("negative file size {file_size}"),
                });
            }
            let file_size = file_size as usize;
            let actual_offset = start_block as u64 * block_size as u64;
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
                id,
                block_size,
                start_block,
                language_id,
                data: &file_data[start..end],
            });
            offset += LUT_ENTRY_64_SIZE;
        }
        Ok(entries)
    }
}
