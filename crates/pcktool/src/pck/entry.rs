//! PCK file entry types.
//!
//! Each entry in a PCK LUT borrows its data from the original file slice.

/// Trait for constructing a file entry from raw LUT fields (32-bit key variant).
pub trait FileEntry<'a, K>: Sized {
    fn from_raw_32(
        id: K,
        block_size: u32,
        start_block: u32,
        language_id: u32,
        data: &'a [u8],
    ) -> Self;
}

/// A sound bank (`.bnk`) entry in a PCK file.
#[derive(Debug, Clone)]
pub struct SoundBankEntry<'a> {
    /// Sound bank ID (FNV1 hash).
    pub id: u32,
    /// Block alignment size.
    pub block_size: u32,
    /// Starting block index in the file.
    pub start_block: u32,
    /// Language ID (0 = SFX / language-independent).
    pub language_id: u32,
    /// Raw BNK data (borrows from the PCK file slice).
    pub data: &'a [u8],
}

impl<'a> FileEntry<'a, u32> for SoundBankEntry<'a> {
    fn from_raw_32(
        id: u32,
        block_size: u32,
        start_block: u32,
        language_id: u32,
        data: &'a [u8],
    ) -> Self {
        Self {
            id,
            block_size,
            start_block,
            language_id,
            data,
        }
    }
}

/// A streaming WEM file entry in a PCK file.
#[derive(Debug, Clone)]
pub struct StreamingFileEntry<'a> {
    /// WEM source ID.
    pub id: u32,
    /// Block alignment size.
    pub block_size: u32,
    /// Starting block index in the file.
    pub start_block: u32,
    /// Language ID.
    pub language_id: u32,
    /// Raw WEM data (borrows from the PCK file slice).
    pub data: &'a [u8],
}

impl<'a> FileEntry<'a, u32> for StreamingFileEntry<'a> {
    fn from_raw_32(
        id: u32,
        block_size: u32,
        start_block: u32,
        language_id: u32,
        data: &'a [u8],
    ) -> Self {
        Self {
            id,
            block_size,
            start_block,
            language_id,
            data,
        }
    }
}

/// An external file entry in a PCK file (64-bit ID).
#[derive(Debug, Clone)]
pub struct ExternalFileEntry<'a> {
    /// External file ID (64-bit).
    pub id: u64,
    /// Block alignment size.
    pub block_size: u32,
    /// Starting block index in the file.
    pub start_block: u32,
    /// Language ID.
    pub language_id: u32,
    /// Raw file data (borrows from the PCK file slice).
    pub data: &'a [u8],
}
