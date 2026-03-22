//! Media source types.

use alloc::vec::Vec;
use crate::error::Result;
use super::super::reader::BinaryReader;
use super::super::writer::BinaryWriter;

#[derive(Debug, Clone)]
pub struct MediaInformation {
    pub source_id: u32,
    pub in_memory_media_size: u32,
    pub source_bits: u8,
}

impl MediaInformation {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            source_id: r.read_u32()?,
            in_memory_media_size: r.read_u32()?,
            source_bits: r.read_u8()?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u32(self.source_id);
        w.write_u32(self.in_memory_media_size);
        w.write_u8(self.source_bits);
    }
    pub fn is_language_specific(&self) -> bool { (self.source_bits & 0x01) != 0 }
    pub fn has_source(&self) -> bool { (self.source_bits & 0x80) != 0 }
}

#[derive(Debug, Clone)]
pub struct BankSourceData {
    pub plugin_id: u32,
    pub stream_type: u8,
    pub media_info: MediaInformation,
    pub plugin_params_size: Option<u32>,
    pub plugin_params: Option<Vec<u8>>,
}

impl BankSourceData {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let plugin_id = r.read_u32()?;
        let stream_type = r.read_u8()?;
        let media_info = MediaInformation::read(r)?;
        let plugin_type_value = plugin_id & 0x000F;
        let (plugin_params_size, plugin_params) = if plugin_type_value == 2 || plugin_type_value == 5 {
            let size = r.read_u32()?;
            let params = if size > 0 {
                Some(r.read_bytes(size as usize)?.to_vec())
            } else {
                None
            };
            (Some(size), params)
        } else {
            (None, None)
        };
        Ok(Self { plugin_id, stream_type, media_info, plugin_params_size, plugin_params })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u32(self.plugin_id);
        w.write_u8(self.stream_type);
        self.media_info.write(w);
        let plugin_type_value = self.plugin_id & 0x000F;
        if plugin_type_value == 2 || plugin_type_value == 5 {
            let size = self.plugin_params.as_ref().map_or(0u32, |p| p.len() as u32);
            w.write_u32(size);
            if let Some(p) = &self.plugin_params {
                w.write_bytes(p);
            }
        }
    }
}

