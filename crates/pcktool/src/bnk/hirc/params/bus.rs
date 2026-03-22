//! Bus parameter types.

use super::super::reader::BinaryReader;
use super::super::writer::BinaryWriter;
use super::node::FxChunk;
use super::props::PropBundle;
use crate::error::Result;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct BusInitialParams {
    pub prop_bundle: PropBundle,
    pub flags1: u8,
    pub flags2: u8,
    pub max_num_instance: u16,
    pub channel_config: u32,
    pub flags3: u8,
}

impl BusInitialParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            prop_bundle: PropBundle::read(r, false, false)?,
            flags1: r.read_u8()?,
            flags2: r.read_u8()?,
            max_num_instance: r.read_u16()?,
            channel_config: r.read_u32()?,
            flags3: r.read_u8()?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        self.prop_bundle.write(w);
        w.write_u8(self.flags1);
        w.write_u8(self.flags2);
        w.write_u16(self.max_num_instance);
        w.write_u32(self.channel_config);
        w.write_u8(self.flags3);
    }
}

#[derive(Debug, Clone)]
pub struct BusInitialFxParams {
    pub bits_fx_bypass: Option<u8>,
    pub fx_chunks: Vec<FxChunk>,
    pub fx_id_0: u32,
    pub is_share_set_0: u8,
}

impl BusInitialFxParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let num_fx = r.read_u8()?;
        let (bits_fx_bypass, fx_chunks) = if num_fx > 0 {
            let bits = r.read_u8()?;
            let mut chunks = Vec::with_capacity(num_fx as usize);
            for _ in 0..num_fx {
                chunks.push(FxChunk::read(r)?);
            }
            (Some(bits), chunks)
        } else {
            (None, Vec::new())
        };
        let fx_id_0 = r.read_u32()?;
        let is_share_set_0 = r.read_u8()?;
        Ok(Self {
            bits_fx_bypass,
            fx_chunks,
            fx_id_0,
            is_share_set_0,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.fx_chunks.len() as u8);
        if let Some(bits) = self.bits_fx_bypass {
            w.write_u8(bits);
            for c in &self.fx_chunks {
                c.write(w);
            }
        }
        w.write_u32(self.fx_id_0);
        w.write_u8(self.is_share_set_0);
    }
}
