//! PCK language string map.
//!
//! The string map stores `(offset, id)` pairs followed by null-terminated UTF-16LE strings.

use alloc::string::String;
use alloc::vec::Vec;
use hashbrown::HashMap;

use crate::error::Result;

/// Parsed string map (language ID → name).
#[derive(Debug, Clone)]
pub struct StringMap {
    pub entries: HashMap<u32, String>,
}

impl StringMap {
    /// Parse a string map from a byte slice.
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut entries = HashMap::new();

        if data.len() < 4 {
            return Ok(Self { entries });
        }

        let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        if count == 0 {
            return Ok(Self { entries });
        }

        // Read (offset, id) pairs — each is 8 bytes
        let mut pairs = Vec::with_capacity(count);
        let mut cursor = 4usize;
        for _ in 0..count {
            if cursor + 8 > data.len() {
                break;
            }
            let offset = u32::from_le_bytes([
                data[cursor],
                data[cursor + 1],
                data[cursor + 2],
                data[cursor + 3],
            ]);
            let id = u32::from_le_bytes([
                data[cursor + 4],
                data[cursor + 5],
                data[cursor + 6],
                data[cursor + 7],
            ]);
            pairs.push((offset, id));
            cursor += 8;
        }

        // Read strings at their offsets (UTF-16LE, null-terminated)
        for (offset, id) in pairs {
            let str_start = offset as usize;
            if str_start >= data.len() {
                continue;
            }
            let name = read_wstring(&data[str_start..]);
            entries.insert(id, name);
        }

        Ok(Self { entries })
    }

    /// Serialize the string map to bytes.
    pub fn write(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Count
        let count = self.entries.len() as u32;
        buf.extend_from_slice(&count.to_le_bytes());

        if count == 0 {
            return buf;
        }

        // Collect entries in deterministic order for reproducible output
        let mut sorted: Vec<_> = self.entries.iter().collect();
        sorted.sort_by_key(|(id, _)| **id);

        // Calculate where strings start: 4 (count) + 8 * count (offset+id pairs)
        let strings_start = 4u32 + 8 * count;

        // First pass: compute string offsets
        let mut string_offsets = Vec::with_capacity(sorted.len());
        let mut current_offset = strings_start;
        for (_, name) in &sorted {
            string_offsets.push(current_offset);
            // Each char is 2 bytes (UTF-16LE) + 2 bytes null terminator
            current_offset += (name.len() as u32 + 1) * 2;
        }

        // Write offset+id pairs
        for (i, (id, _)) in sorted.iter().enumerate() {
            buf.extend_from_slice(&string_offsets[i].to_le_bytes());
            buf.extend_from_slice(&id.to_le_bytes());
        }

        // Write strings (UTF-16LE, null-terminated)
        for (_, name) in &sorted {
            for ch in name.encode_utf16() {
                buf.extend_from_slice(&ch.to_le_bytes());
            }
            buf.extend_from_slice(&0u16.to_le_bytes()); // null terminator
        }

        // Pad to 4-byte alignment
        let padding = (4 - buf.len() % 4) % 4;
        buf.extend(core::iter::repeat_n(0u8, padding));

        buf
    }
}

/// Read a null-terminated UTF-16LE string from a byte slice.
fn read_wstring(data: &[u8]) -> String {
    let mut chars = Vec::new();
    let mut i = 0;
    while i + 1 < data.len() {
        let ch = u16::from_le_bytes([data[i], data[i + 1]]);
        if ch == 0 {
            break;
        }
        chars.push(ch);
        i += 2;
    }
    String::from_utf16_lossy(&chars)
}
