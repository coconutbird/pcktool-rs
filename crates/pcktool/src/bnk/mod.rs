//! BNK (Wwise soundbank) file format support.
//!
//! A BNK file is a sequence of tagged chunks:
//! ```text
//! [chunk_tag: u32][chunk_size: u32][chunk_data: [u8; chunk_size]]
//! ```
//!
//! Known chunk types:
//! - `BKHD` — Bank header (version, bank ID, language, project ID)
//! - `DIDX` — Data index (media ID → offset/size table)
//! - `DATA` — Embedded media data (WEM files)
//! - `HIRC` — Hierarchy (events, actions, sounds, containers)
//! - `STID` — String map (bank ID → name)
//! - `STMG` — State manager (global settings)
//! - `ENVS` — Environment settings
//! - `FXPR` — FX parameters
//! - `PLAT` — Platform-specific data
//! - `INIT` — Plugin initialization

mod chunks;
pub mod hirc;

pub use chunks::{BankHeader, MediaEntry, MediaIndex};

use alloc::format;
use alloc::vec::Vec;
use hashbrown::HashMap;

use crate::error::{Error, Result};
use zerocopy::{FromBytes, IntoBytes, little_endian as le};

// Chunk tag constants
/// Known Wwise BNK chunk tags.
pub mod tags {
    use crate::hash::fourcc;

    pub const BKHD: u32 = fourcc(b"BKHD");
    pub const DIDX: u32 = fourcc(b"DIDX");
    pub const DATA: u32 = fourcc(b"DATA");
    pub const HIRC: u32 = fourcc(b"HIRC");
    pub const STID: u32 = fourcc(b"STID");
    pub const STMG: u32 = fourcc(b"STMG");
    pub const FXPR: u32 = fourcc(b"FXPR");
    pub const ENVS: u32 = fourcc(b"ENVS");
    pub const PLAT: u32 = fourcc(b"PLAT");
    pub const INIT: u32 = fourcc(b"INIT");
}

use tags::*;

/// On-disk chunk header.
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes)]
#[repr(C, packed)]
pub struct ChunkHeader {
    pub tag: le::U32,
    pub size: le::U32,
}

/// A parsed Wwise SoundBank.
#[derive(Debug)]
pub struct SoundBank<'a> {
    /// Bank header fields.
    pub header: BankHeader,
    /// Embedded media: source ID → WEM data slice.
    pub media: HashMap<u32, &'a [u8]>,
    /// HIRC chunk raw data (preserved for round-tripping).
    pub hirc_data: Option<&'a [u8]>,
    /// All chunks in order, for round-trip serialization.
    /// Each is (tag, data_slice). Known chunks are also parsed above.
    chunks: Vec<Chunk<'a>>,
}

/// A raw chunk reference.
#[derive(Debug, Clone)]
struct Chunk<'a> {
    tag: u32,
    data: &'a [u8],
}

impl<'a> SoundBank<'a> {
    /// Parse a soundbank from a byte slice.
    pub fn parse(data: &'a [u8]) -> Result<Self> {
        let mut header = BankHeader::default();
        let mut media_index: Vec<MediaIndex> = Vec::new();
        let mut media = HashMap::new();
        let mut hirc_data = None;
        let mut chunks = Vec::new();

        let chunk_hdr_size = core::mem::size_of::<ChunkHeader>();
        let mut cursor = 0usize;

        while cursor + chunk_hdr_size <= data.len() {
            let ch = ChunkHeader::read_from_bytes(&data[cursor..cursor + chunk_hdr_size]).map_err(
                |e| Error::ZeroCopyCast {
                    offset: cursor as u64,
                    reason: format!("{e:?}"),
                },
            )?;

            let tag = ch.tag.get();
            let size = ch.size.get() as usize;
            let chunk_start = cursor + chunk_hdr_size;
            let chunk_end = chunk_start + size;

            if chunk_end > data.len() {
                return Err(Error::UnexpectedEof {
                    offset: cursor as u64,
                    needed: (chunk_hdr_size + size) as u64,
                    available: (data.len() - cursor) as u64,
                });
            }

            let chunk_data = &data[chunk_start..chunk_end];
            chunks.push(Chunk {
                tag,
                data: chunk_data,
            });

            match tag {
                BKHD => {
                    header = BankHeader::parse(chunk_data)?;
                }
                DIDX => {
                    media_index = MediaIndex::parse_all(chunk_data)?;
                }
                DATA => {
                    // Use previously parsed DIDX to slice out individual media
                    for idx in &media_index {
                        let start = idx.offset.get() as usize;
                        let end = start + idx.size.get() as usize;
                        if end <= chunk_data.len() {
                            media.insert(idx.id.get(), &chunk_data[start..end]);
                        }
                    }
                }
                HIRC => {
                    hirc_data = Some(chunk_data);
                }
                // All other chunks are preserved in `chunks` for round-tripping
                _ => {}
            }

            cursor = chunk_end;
        }

        Ok(SoundBank {
            header,
            media,
            hirc_data,
            chunks,
        })
    }

    /// Find embedded media by source ID.
    pub fn find_media(&self, source_id: u32) -> Option<&'a [u8]> {
        self.media.get(&source_id).copied()
    }

    /// Serialize the soundbank back to bytes.
    ///
    /// Chunks are written in the same order they were parsed. If media was modified
    /// via [`SoundBankOwned`], use that type's `to_bytes()` instead.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        for chunk in &self.chunks {
            buf.extend_from_slice(&chunk.tag.to_le_bytes());
            buf.extend_from_slice(&(chunk.data.len() as u32).to_le_bytes());
            buf.extend_from_slice(chunk.data);
        }
        buf
    }
}

/// An owned soundbank that supports media replacement and re-serialization.
#[derive(Debug, Clone)]
pub struct SoundBankOwned {
    /// Bank header.
    pub header: BankHeader,
    /// Embedded media: source ID → owned WEM data.
    pub media: HashMap<u32, Vec<u8>>,
    /// HIRC chunk raw data.
    pub hirc_data: Option<Vec<u8>>,
    /// All chunks in original order: (tag, owned data).
    /// DIDX/DATA are regenerated from `media` on write.
    chunks: Vec<OwnedChunk>,
}

#[derive(Debug, Clone)]
struct OwnedChunk {
    tag: u32,
    data: Vec<u8>,
}

impl SoundBankOwned {
    /// Create an owned copy from a borrowed soundbank.
    pub fn from_borrowed(bank: &SoundBank<'_>) -> Self {
        let media: HashMap<u32, Vec<u8>> = bank
            .media
            .iter()
            .map(|(&id, &data)| (id, data.into()))
            .collect();

        let hirc_data = bank.hirc_data.map(|d| d.into());

        let chunks = bank
            .chunks
            .iter()
            .map(|c| OwnedChunk {
                tag: c.tag,
                data: c.data.into(),
            })
            .collect();

        Self {
            header: bank.header.clone(),
            media,
            hirc_data,
            chunks,
        }
    }

    /// Replace embedded media for a given source ID.
    pub fn replace_media(&mut self, source_id: u32, data: Vec<u8>) -> bool {
        if self.media.contains_key(&source_id) {
            self.media.insert(source_id, data);
            true
        } else {
            false
        }
    }

    /// Serialize to bytes, regenerating DIDX/DATA from current media.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        for chunk in &self.chunks {
            match chunk.tag {
                DIDX => {
                    // Regenerate DIDX from media
                    let didx = self.build_didx();
                    buf.extend_from_slice(&DIDX.to_le_bytes());
                    buf.extend_from_slice(&(didx.len() as u32).to_le_bytes());
                    buf.extend_from_slice(&didx);
                }
                DATA => {
                    // Regenerate DATA from media
                    let data = self.build_data();
                    buf.extend_from_slice(&DATA.to_le_bytes());
                    buf.extend_from_slice(&(data.len() as u32).to_le_bytes());
                    buf.extend_from_slice(&data);
                }
                BKHD => {
                    let hdr = self.header.to_bytes();
                    buf.extend_from_slice(&BKHD.to_le_bytes());
                    buf.extend_from_slice(&(hdr.len() as u32).to_le_bytes());
                    buf.extend_from_slice(&hdr);
                }
                _ => {
                    buf.extend_from_slice(&chunk.tag.to_le_bytes());
                    buf.extend_from_slice(&(chunk.data.len() as u32).to_le_bytes());
                    buf.extend_from_slice(&chunk.data);
                }
            }
        }
        buf
    }

    fn build_didx(&self) -> Vec<u8> {
        // Sort by ID for deterministic output
        let mut ids: Vec<u32> = self.media.keys().copied().collect();
        ids.sort();

        let mut didx = Vec::with_capacity(ids.len() * 12);
        let mut offset = 0u32;
        for &id in &ids {
            let size = self.media[&id].len() as u32;
            didx.extend_from_slice(&id.to_le_bytes());
            didx.extend_from_slice(&offset.to_le_bytes());
            didx.extend_from_slice(&size.to_le_bytes());
            offset += size;
        }
        didx
    }

    fn build_data(&self) -> Vec<u8> {
        let mut ids: Vec<u32> = self.media.keys().copied().collect();
        ids.sort();

        let total: usize = ids.iter().map(|id| self.media[id].len()).sum();
        let mut data = Vec::with_capacity(total);
        for &id in &ids {
            data.extend_from_slice(&self.media[&id]);
        }
        data
    }
}
