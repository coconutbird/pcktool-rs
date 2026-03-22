//! Plugin types and parameters.

use alloc::vec::Vec;
use crate::error::Result;
use super::super::reader::BinaryReader;
use super::super::writer::BinaryWriter;

/// Known Wwise plugin IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PluginId(pub u32);

impl PluginId {
    pub const NONE: Self = Self(0x00000000);
    pub const WWISE_SILENCE: Self = Self(0x00650002);
    pub const WWISE_DELAY: Self = Self(0x006A0003);
    pub const WWISE_COMPRESSOR: Self = Self(0x006C0003);
}

/// Opaque plugin parameter block.
#[derive(Debug, Clone)]
pub struct PluginParam {
    pub param_block: Vec<u8>,
}

impl PluginParam {
    pub fn read(r: &mut BinaryReader, size: u32) -> Result<Self> {
        let param_block = r.read_bytes(size as usize)?.to_vec();
        Ok(Self { param_block })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_bytes(&self.param_block);
    }
}

#[derive(Debug, Clone)]
pub struct FxSrcSilenceParams {
    pub duration: f32,
    pub randomized_length_minus: f32,
    pub randomized_length_plus: f32,
}

impl FxSrcSilenceParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            duration: r.read_f32()?,
            randomized_length_minus: r.read_f32()?,
            randomized_length_plus: r.read_f32()?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_f32(self.duration);
        w.write_f32(self.randomized_length_minus);
        w.write_f32(self.randomized_length_plus);
    }
}

#[derive(Debug, Clone)]
pub struct DelayFxParams {
    pub non_rtpc_delay_time: f32,
    pub rtpc_feedback: f32,
    pub rtpc_wet_dry_mix: f32,
    pub rtpc_output_level: f32,
    pub rtpc_feedback_enabled: u8,
    pub non_rtpc_process_lfe: u8,
}

impl DelayFxParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            non_rtpc_delay_time: r.read_f32()?,
            rtpc_feedback: r.read_f32()?,
            rtpc_wet_dry_mix: r.read_f32()?,
            rtpc_output_level: r.read_f32()?,
            rtpc_feedback_enabled: r.read_u8()?,
            non_rtpc_process_lfe: r.read_u8()?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_f32(self.non_rtpc_delay_time);
        w.write_f32(self.rtpc_feedback);
        w.write_f32(self.rtpc_wet_dry_mix);
        w.write_f32(self.rtpc_output_level);
        w.write_u8(self.rtpc_feedback_enabled);
        w.write_u8(self.non_rtpc_process_lfe);
    }
}

