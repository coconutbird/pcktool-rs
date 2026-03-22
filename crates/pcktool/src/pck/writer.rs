//! PCK file writer — serializes a PCK to `Vec<u8>`.

use alloc::vec::Vec;
use hashbrown::HashMap;

use alloc::string::String;

use super::{AKPK_TAG, PCK_VERSION, StringMap};

/// An owned entry for writing into a PCK file.
#[derive(Debug, Clone)]
pub struct WriteEntry<K: Copy> {
    pub id: K,
    pub block_size: u32,
    pub language_id: u32,
    pub data: Vec<u8>,
}

/// Builder for writing a PCK file to `Vec<u8>`.
#[derive(Debug, Clone)]
pub struct PckWriter {
    pub languages: HashMap<u32, String>,
    pub sound_banks: Vec<WriteEntry<u32>>,
    pub streaming_files: Vec<WriteEntry<u32>>,
    pub external_files: Vec<WriteEntry<u64>>,
}

impl PckWriter {
    /// Create a new empty writer.
    pub fn new() -> Self {
        Self {
            languages: HashMap::new(),
            sound_banks: Vec::new(),
            streaming_files: Vec::new(),
            external_files: Vec::new(),
        }
    }

    /// Serialize the PCK file to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Tag
        buf.extend_from_slice(&AKPK_TAG.to_le_bytes());

        // Placeholder for header_size (offset 4)
        let header_size_pos = buf.len();
        buf.extend_from_slice(&0u32.to_le_bytes());

        // Version
        buf.extend_from_slice(&PCK_VERSION.to_le_bytes());

        // Placeholders for section sizes (offset 12..28)
        let section_sizes_pos = buf.len();
        buf.extend_from_slice(&0u32.to_le_bytes()); // language_map_size
        buf.extend_from_slice(&0u32.to_le_bytes()); // sound_banks_lut_size
        buf.extend_from_slice(&0u32.to_le_bytes()); // streaming_files_lut_size
        buf.extend_from_slice(&0u32.to_le_bytes()); // external_files_lut_size

        // Write language map
        let lang_map = StringMap {
            entries: self.languages.clone(),
        };
        let lang_bytes = lang_map.write();
        let lang_map_size = lang_bytes.len() as u32;
        buf.extend_from_slice(&lang_bytes);

        // Calculate LUT sizes
        let banks_lut_size = lut_size_32(self.sound_banks.len());
        let stm_lut_size = lut_size_32(self.streaming_files.len());
        let ext_lut_size = lut_size_64(self.external_files.len());

        // header_size = everything after tag(4) + header_size_field(4)
        let header_size = (buf.len() - 8 + banks_lut_size + stm_lut_size + ext_lut_size) as u32;
        let data_start = 8 + header_size as usize;

        // Calculate start blocks for all entries
        let mut current_offset = data_start;
        let bank_blocks = calc_start_blocks_32(&self.sound_banks, &mut current_offset);
        let stm_blocks = calc_start_blocks_32(&self.streaming_files, &mut current_offset);
        let ext_blocks = calc_start_blocks_64(&self.external_files, &mut current_offset);

        // Write LUT headers
        write_lut_32(&mut buf, &self.sound_banks, &bank_blocks);
        write_lut_32(&mut buf, &self.streaming_files, &stm_blocks);
        write_lut_64(&mut buf, &self.external_files, &ext_blocks);

        // Write file data at correct offsets
        let total_size = current_offset;
        buf.resize(total_size, 0);
        write_data_32(&mut buf, &self.sound_banks, &bank_blocks);
        write_data_32(&mut buf, &self.streaming_files, &stm_blocks);
        write_data_64(&mut buf, &self.external_files, &ext_blocks);

        // Patch header sizes
        buf[header_size_pos..header_size_pos + 4].copy_from_slice(&header_size.to_le_bytes());
        buf[section_sizes_pos..section_sizes_pos + 4].copy_from_slice(&lang_map_size.to_le_bytes());
        buf[section_sizes_pos + 4..section_sizes_pos + 8]
            .copy_from_slice(&(banks_lut_size as u32).to_le_bytes());
        buf[section_sizes_pos + 8..section_sizes_pos + 12]
            .copy_from_slice(&(stm_lut_size as u32).to_le_bytes());
        buf[section_sizes_pos + 12..section_sizes_pos + 16]
            .copy_from_slice(&(ext_lut_size as u32).to_le_bytes());

        buf
    }
}

impl Default for PckWriter {
    fn default() -> Self {
        Self::new()
    }
}

fn lut_size_32(count: usize) -> usize {
    4 + count * 20 // count(4) + entries * (id:4 + block:4 + size:4 + start:4 + lang:4)
}

fn lut_size_64(count: usize) -> usize {
    4 + count * 24 // count(4) + entries * (id:8 + block:4 + size:4 + start:4 + lang:4)
}

fn align_up(offset: usize, alignment: u32) -> usize {
    if alignment <= 1 {
        return offset;
    }
    let a = alignment as usize;
    offset.div_ceil(a) * a
}

/// Returns (start_block, block_size) for each entry.
fn calc_start_blocks_32(
    entries: &[WriteEntry<u32>],
    current_offset: &mut usize,
) -> Vec<(u32, u32)> {
    let mut blocks = Vec::with_capacity(entries.len());
    for entry in entries {
        *current_offset = align_up(*current_offset, entry.block_size);
        let start_block = if entry.block_size > 1 {
            (*current_offset / entry.block_size as usize) as u32
        } else {
            *current_offset as u32
        };
        blocks.push((start_block, entry.block_size));
        *current_offset += entry.data.len();
    }
    blocks
}

fn calc_start_blocks_64(
    entries: &[WriteEntry<u64>],
    current_offset: &mut usize,
) -> Vec<(u32, u32)> {
    let mut blocks = Vec::with_capacity(entries.len());
    for entry in entries {
        *current_offset = align_up(*current_offset, entry.block_size);
        let start_block = if entry.block_size > 1 {
            (*current_offset / entry.block_size as usize) as u32
        } else {
            *current_offset as u32
        };
        blocks.push((start_block, entry.block_size));
        *current_offset += entry.data.len();
    }
    blocks
}

fn write_lut_32(buf: &mut Vec<u8>, entries: &[WriteEntry<u32>], blocks: &[(u32, u32)]) {
    buf.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    for (i, entry) in entries.iter().enumerate() {
        buf.extend_from_slice(&entry.id.to_le_bytes());
        buf.extend_from_slice(&entry.block_size.to_le_bytes());
        buf.extend_from_slice(&(entry.data.len() as i32).to_le_bytes());
        buf.extend_from_slice(&blocks[i].0.to_le_bytes());
        buf.extend_from_slice(&entry.language_id.to_le_bytes());
    }
}

fn write_lut_64(buf: &mut Vec<u8>, entries: &[WriteEntry<u64>], blocks: &[(u32, u32)]) {
    buf.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    for (i, entry) in entries.iter().enumerate() {
        buf.extend_from_slice(&entry.id.to_le_bytes());
        buf.extend_from_slice(&entry.block_size.to_le_bytes());
        buf.extend_from_slice(&(entry.data.len() as i32).to_le_bytes());
        buf.extend_from_slice(&blocks[i].0.to_le_bytes());
        buf.extend_from_slice(&entry.language_id.to_le_bytes());
    }
}

fn write_data_32(buf: &mut [u8], entries: &[WriteEntry<u32>], blocks: &[(u32, u32)]) {
    for (i, entry) in entries.iter().enumerate() {
        let byte_offset = blocks[i].0 as usize * blocks[i].1 as usize;
        buf[byte_offset..byte_offset + entry.data.len()].copy_from_slice(&entry.data);
    }
}

fn write_data_64(buf: &mut [u8], entries: &[WriteEntry<u64>], blocks: &[(u32, u32)]) {
    for (i, entry) in entries.iter().enumerate() {
        let byte_offset = blocks[i].0 as usize * blocks[i].1 as usize;
        buf[byte_offset..byte_offset + entry.data.len()].copy_from_slice(&entry.data);
    }
}
