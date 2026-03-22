//! Error types for pcktool.

use alloc::string::String;
use core::fmt;

/// Result alias using [`Error`].
pub type Result<T> = core::result::Result<T, Error>;

/// Errors that can occur when parsing or writing Wwise audio packages.
#[derive(Debug)]
pub enum Error {
    /// The file header magic bytes are invalid.
    InvalidMagic { expected: u32, actual: u32 },

    /// The file version is unsupported.
    UnsupportedVersion(u32),

    /// The file is truncated or a size field points past EOF.
    UnexpectedEof {
        offset: u64,
        needed: u64,
        available: u64,
    },

    /// A LUT entry references invalid data.
    InvalidEntry { index: u32, reason: String },

    /// A BNK chunk could not be parsed.
    InvalidChunk {
        tag: String,
        offset: u64,
        reason: String,
    },

    /// A WEM source ID was not found.
    WemNotFound(u32),

    /// A soundbank ID was not found.
    SoundBankNotFound(u32),

    /// The data failed zerocopy validation.
    ZeroCopyCast { offset: u64, reason: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic { expected, actual } => {
                write!(
                    f,
                    "invalid magic: expected 0x{expected:08X}, got 0x{actual:08X}"
                )
            }
            Self::UnsupportedVersion(v) => write!(f, "unsupported version: 0x{v:X}"),
            Self::UnexpectedEof {
                offset,
                needed,
                available,
            } => {
                write!(
                    f,
                    "unexpected end of data at offset 0x{offset:X} (need {needed} bytes, have {available})"
                )
            }
            Self::InvalidEntry { index, reason } => {
                write!(f, "invalid entry at index {index}: {reason}")
            }
            Self::InvalidChunk {
                tag,
                offset,
                reason,
            } => {
                write!(f, "invalid chunk '{tag}' at offset 0x{offset:X}: {reason}")
            }
            Self::WemNotFound(id) => write!(f, "WEM source ID 0x{id:08X} not found"),
            Self::SoundBankNotFound(id) => write!(f, "soundbank ID 0x{id:08X} not found"),
            Self::ZeroCopyCast { offset, reason } => {
                write!(f, "zerocopy cast failed at offset 0x{offset:X}: {reason}")
            }
        }
    }
}

impl core::error::Error for Error {}
