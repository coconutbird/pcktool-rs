//! Cursor-based binary reader for `no_std` HIRC parsing.

use crate::error::{Error, Result};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// A cursor over a byte slice for sequential binary reads.
#[derive(Debug, Clone)]
pub struct BinaryReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> BinaryReader<'a> {
    /// Create a new reader over `data`.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Current read position.
    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Remaining bytes.
    #[inline]
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    /// Total length of underlying data.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Whether the reader is at or past the end.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    fn check(&self, n: usize) -> Result<()> {
        if self.pos + n > self.data.len() {
            Err(Error::UnexpectedEof {
                offset: self.pos as u64,
                needed: n as u64,
                available: self.remaining() as u64,
            })
        } else {
            Ok(())
        }
    }

    /// Read a `u8`.
    pub fn read_u8(&mut self) -> Result<u8> {
        self.check(1)?;
        let v = self.data[self.pos];
        self.pos += 1;
        Ok(v)
    }

    /// Read an `i8`.
    pub fn read_i8(&mut self) -> Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    /// Read a little-endian `u16`.
    pub fn read_u16(&mut self) -> Result<u16> {
        self.check(2)?;
        let v = u16::from_le_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Ok(v)
    }

    /// Read a little-endian `i16`.
    pub fn read_i16(&mut self) -> Result<i16> {
        Ok(self.read_u16()? as i16)
    }

    /// Read a little-endian `u32`.
    pub fn read_u32(&mut self) -> Result<u32> {
        self.check(4)?;
        let v = u32::from_le_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Ok(v)
    }

    /// Read a little-endian `i32`.
    pub fn read_i32(&mut self) -> Result<i32> {
        Ok(self.read_u32()? as i32)
    }

    /// Read a little-endian `f32`.
    pub fn read_f32(&mut self) -> Result<f32> {
        Ok(f32::from_bits(self.read_u32()?))
    }

    /// Read a little-endian `f64` / `double`.
    pub fn read_f64(&mut self) -> Result<f64> {
        self.check(8)?;
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.data[self.pos..self.pos + 8]);
        self.pos += 8;
        Ok(f64::from_le_bytes(bytes))
    }

    /// Read `n` raw bytes.
    pub fn read_bytes(&mut self, n: usize) -> Result<&'a [u8]> {
        self.check(n)?;
        let slice = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    /// Read a null-terminated UTF-8 string of known byte length (including null).
    pub fn read_string(&mut self, byte_len: usize) -> Result<String> {
        let bytes = self.read_bytes(byte_len)?;
        // Strip trailing null(s)
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(byte_len);
        String::from_utf8(bytes[..end].to_vec()).map_err(|_| Error::InvalidChunk {
            tag: "HIRC".to_string(),
            offset: (self.pos - byte_len) as u64,
            reason: "invalid UTF-8 string".to_string(),
        })
    }

    /// Read a list with a `u32` count prefix.
    pub fn read_list_u32<T, F>(&mut self, f: F) -> Result<Vec<T>>
    where
        F: Fn(&mut Self) -> Result<T>,
    {
        let count = self.read_u32()? as usize;
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            items.push(f(self)?);
        }
        Ok(items)
    }

    /// Read a list with a `u16` count prefix.
    pub fn read_list_u16<T, F>(&mut self, f: F) -> Result<Vec<T>>
    where
        F: Fn(&mut Self) -> Result<T>,
    {
        let count = self.read_u16()? as usize;
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            items.push(f(self)?);
        }
        Ok(items)
    }
}
