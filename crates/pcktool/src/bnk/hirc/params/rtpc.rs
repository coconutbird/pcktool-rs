//! RTPC (Real-Time Parameter Control) types.

use alloc::vec::Vec;
use crate::error::Result;
use super::super::reader::BinaryReader;
use super::super::writer::BinaryWriter;

/// A point on an RTPC graph curve.
#[derive(Debug, Clone, Copy)]
pub struct RtpcGraphPoint {
    pub from: f32,
    pub to: f32,
    pub interpolation: u32,
}

impl RtpcGraphPoint {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            from: r.read_f32()?,
            to: r.read_f32()?,
            interpolation: r.read_u32()?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_f32(self.from);
        w.write_f32(self.to);
        w.write_u32(self.interpolation);
    }
}

#[derive(Debug, Clone)]
pub struct RtpcManager {
    pub rtpc_id: u32,
    pub rtpc_type: u8,
    pub rtpc_accum: u8,
    pub param_id: u8,
    pub rtpc_curve_id: u32,
    pub scaling: u8,
    pub graph_points: Vec<RtpcGraphPoint>,
}

impl RtpcManager {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let rtpc_id = r.read_u32()?;
        let rtpc_type = r.read_u8()?;
        let rtpc_accum = r.read_u8()?;
        let param_id = r.read_u8()?;
        let rtpc_curve_id = r.read_u32()?;
        let scaling = r.read_u8()?;
        let graph_points = r.read_list_u16(RtpcGraphPoint::read)?;
        Ok(Self { rtpc_id, rtpc_type, rtpc_accum, param_id, rtpc_curve_id, scaling, graph_points })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u32(self.rtpc_id);
        w.write_u8(self.rtpc_type);
        w.write_u8(self.rtpc_accum);
        w.write_u8(self.param_id);
        w.write_u32(self.rtpc_curve_id);
        w.write_u8(self.scaling);
        w.write_list_u16(&self.graph_points, |w, p| p.write(w));
    }
}

#[derive(Debug, Clone, Default)]
pub struct InitialRtpc {
    pub rtpc_managers: Vec<RtpcManager>,
}

impl InitialRtpc {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let managers = r.read_list_u16(RtpcManager::read)?;
        Ok(Self { rtpc_managers: managers })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_list_u16(&self.rtpc_managers, |w, m| m.write(w));
    }
}

#[derive(Debug, Clone)]
pub struct ConversionTable {
    pub scaling: u8,
    pub points: Vec<RtpcGraphPoint>,
}

impl ConversionTable {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let scaling = r.read_u8()?;
        let points = r.read_list_u16(RtpcGraphPoint::read)?;
        Ok(Self { scaling, points })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.scaling);
        w.write_list_u16(&self.points, |w, p| p.write(w));
    }
}

