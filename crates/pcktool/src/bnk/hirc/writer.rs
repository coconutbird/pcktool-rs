//! Cursor-based binary writer for `no_std` HIRC serialization.

use alloc::vec::Vec;

/// A sequential binary writer into a `Vec<u8>`.
#[derive(Debug, Clone)]
pub struct BinaryWriter {
    buf: Vec<u8>,
}

impl BinaryWriter {
    /// Create a new writer.
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    /// Create a new writer with pre-allocated capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Self { buf: Vec::with_capacity(cap) }
    }

    /// Current write position (length of buffer so far).
    #[inline]
    pub fn position(&self) -> usize {
        self.buf.len()
    }

    /// Consume the writer and return the underlying buffer.
    pub fn into_inner(self) -> Vec<u8> {
        self.buf
    }

    /// Get a reference to the underlying buffer.
    pub fn as_slice(&self) -> &[u8] {
        &self.buf
    }

    /// Write a `u8`.
    pub fn write_u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    /// Write an `i8`.
    pub fn write_i8(&mut self, v: i8) {
        self.buf.push(v as u8);
    }

    /// Write a little-endian `u16`.
    pub fn write_u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    /// Write a little-endian `i16`.
    pub fn write_i16(&mut self, v: i16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    /// Write a little-endian `u32`.
    pub fn write_u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    /// Write a little-endian `i32`.
    pub fn write_i32(&mut self, v: i32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    /// Write a little-endian `f32`.
    pub fn write_f32(&mut self, v: f32) {
        self.buf.extend_from_slice(&v.to_bits().to_le_bytes());
    }

    /// Write a little-endian `f64`.
    pub fn write_f64(&mut self, v: f64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    /// Write raw bytes.
    pub fn write_bytes(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
    }

    /// Write a null-terminated string.
    pub fn write_string(&mut self, s: &str) {
        self.buf.extend_from_slice(s.as_bytes());
        self.buf.push(0);
    }

    /// Write a list with a `u32` count prefix.
    pub fn write_list_u32<T, F>(&mut self, items: &[T], f: F)
    where
        F: Fn(&mut Self, &T),
    {
        self.write_u32(items.len() as u32);
        for item in items {
            f(self, item);
        }
    }

    /// Write a list with a `u16` count prefix.
    pub fn write_list_u16<T, F>(&mut self, items: &[T], f: F)
    where
        F: Fn(&mut Self, &T),
    {
        self.write_u16(items.len() as u16);
        for item in items {
            f(self, item);
        }
    }

    /// Patch a `u32` value at a specific offset in the buffer.
    pub fn patch_u32(&mut self, offset: usize, v: u32) {
        let bytes = v.to_le_bytes();
        self.buf[offset..offset + 4].copy_from_slice(&bytes);
    }
}

