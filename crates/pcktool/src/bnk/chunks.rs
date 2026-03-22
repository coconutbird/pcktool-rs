//! BNK chunk structures.

use alloc::format;
use alloc::vec::Vec;

use crate::error::{Error, Result};
use zerocopy::{FromBytes, IntoBytes, little_endian as le};

/// BKHD (Bank Header) parsed fields.
#[derive(Debug, Clone, Default)]
pub struct BankHeader {
    /// Wwise version (typically 0x71 for Halo Wars DE).
    pub version: u32,
    /// Unique soundbank ID.
    pub id: u32,
    /// Language ID (0 = SFX).
    pub language_id: u32,
    /// Feedback-in-bank flag.
    pub feedback_in_bank: u32,
    /// Project ID.
    pub project_id: u32,
    /// Any extra padding bytes after the standard 20-byte header.
    pub padding: Vec<u8>,
}

/// On-disk BKHD layout (first 20 bytes).
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes)]
#[repr(C, packed)]
pub struct RawBankHeader {
    pub version: le::U32,
    pub id: le::U32,
    pub language_id: le::U32,
    pub feedback_in_bank: le::U32,
    pub project_id: le::U32,
}

impl BankHeader {
    /// Parse from BKHD chunk data.
    pub fn parse(data: &[u8]) -> Result<Self> {
        let raw_size = core::mem::size_of::<RawBankHeader>();
        if data.len() < raw_size {
            return Err(Error::InvalidChunk {
                tag: alloc::string::String::from("BKHD"),
                offset: 0,
                reason: format!("need {raw_size} bytes, have {}", data.len()),
            });
        }

        let raw = RawBankHeader::read_from_bytes(&data[..raw_size])
            .map_err(|e| Error::ZeroCopyCast {
                offset: 0,
                reason: format!("{e:?}"),
            })?;

        let padding = if data.len() > raw_size {
            data[raw_size..].into()
        } else {
            Vec::new()
        };

        Ok(Self {
            version: raw.version.get(),
            id: raw.id.get(),
            language_id: raw.language_id.get(),
            feedback_in_bank: raw.feedback_in_bank.get(),
            project_id: raw.project_id.get(),
            padding,
        })
    }

    /// Whether feedback is enabled (affects HIRC NodeBaseParams parsing).
    pub fn has_feedback(&self) -> bool {
        (self.feedback_in_bank & 1) != 0
    }

    /// Serialize to bytes (for writing BKHD chunk data, without the chunk header).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(20 + self.padding.len());
        buf.extend_from_slice(&self.version.to_le_bytes());
        buf.extend_from_slice(&self.id.to_le_bytes());
        buf.extend_from_slice(&self.language_id.to_le_bytes());
        buf.extend_from_slice(&self.feedback_in_bank.to_le_bytes());
        buf.extend_from_slice(&self.project_id.to_le_bytes());
        buf.extend_from_slice(&self.padding);
        buf
    }
}

/// DIDX media index entry (12 bytes each).
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes)]
#[repr(C, packed)]
pub struct MediaIndex {
    /// WEM source ID.
    pub id: le::U32,
    /// Offset within the DATA chunk.
    pub offset: le::U32,
    /// Size in bytes.
    pub size: le::U32,
}

/// Convenience accessors that return native types.
impl MediaIndex {
    /// Parse all DIDX entries from chunk data.
    pub fn parse_all(data: &[u8]) -> Result<Vec<Self>> {
        let entry_size = core::mem::size_of::<Self>();
        let count = data.len() / entry_size;
        let mut entries = Vec::with_capacity(count);

        for i in 0..count {
            let offset = i * entry_size;
            let entry = Self::read_from_bytes(&data[offset..offset + entry_size])
                .map_err(|e| Error::ZeroCopyCast {
                    offset: offset as u64,
                    reason: format!("{e:?}"),
                })?;
            entries.push(entry);
        }

        Ok(entries)
    }
}

/// A single embedded media entry (ID + owned data) for writing.
#[derive(Debug, Clone)]
pub struct MediaEntry {
    pub id: u32,
    pub data: Vec<u8>,
}

