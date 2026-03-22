//! Bus, State, and Attenuation item values.

use alloc::vec::Vec;

use crate::error::Result;
use super::super::params::*;
use super::super::reader::BinaryReader;
use super::super::writer::BinaryWriter;

// ─── Bus ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DuckInfo {
    pub bus_id: u32,
    pub duck_volume: f32,
    pub fade_out_time: i32,
    pub fade_in_time: i32,
    pub fade_curve: u8,
    pub target_prop: u8,
}

impl DuckInfo {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            bus_id: r.read_u32()?,
            duck_volume: r.read_f32()?,
            fade_out_time: r.read_i32()?,
            fade_in_time: r.read_i32()?,
            fade_curve: r.read_u8()?,
            target_prop: r.read_u8()?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u32(self.bus_id);
        w.write_f32(self.duck_volume);
        w.write_i32(self.fade_out_time);
        w.write_i32(self.fade_in_time);
        w.write_u8(self.fade_curve);
        w.write_u8(self.target_prop);
    }
}

#[derive(Debug, Clone)]
pub struct BusValues {
    pub override_bus_id: u32,
    pub bus_initial_params: BusInitialParams,
    pub recovery_time: i32,
    pub max_duck_volume: f32,
    pub ducks: Vec<DuckInfo>,
    pub bus_initial_fx_params: BusInitialFxParams,
    pub override_attachment_params: u8,
    pub initial_rtpc: InitialRtpc,
    pub state_chunk: StateChunk,
    pub has_feedback: bool,
    pub feedback_bus_id: u32,
}

impl BusValues {
    pub fn read(r: &mut BinaryReader, has_feedback: bool) -> Result<Self> {
        let override_bus_id = r.read_u32()?;
        let bus_initial_params = BusInitialParams::read(r)?;
        let recovery_time = r.read_i32()?;
        let max_duck_volume = r.read_f32()?;
        let ducks = r.read_list_u32(DuckInfo::read)?;
        let bus_initial_fx_params = BusInitialFxParams::read(r)?;
        let override_attachment_params = r.read_u8()?;
        let initial_rtpc = InitialRtpc::read(r)?;
        let state_chunk = StateChunk::read(r)?;
        let feedback_bus_id = if has_feedback { r.read_u32()? } else { 0 };
        Ok(Self {
            override_bus_id, bus_initial_params, recovery_time, max_duck_volume,
            ducks, bus_initial_fx_params, override_attachment_params,
            initial_rtpc, state_chunk, has_feedback, feedback_bus_id,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u32(self.override_bus_id);
        self.bus_initial_params.write(w);
        w.write_i32(self.recovery_time);
        w.write_f32(self.max_duck_volume);
        w.write_list_u32(&self.ducks, |w, d| d.write(w));
        self.bus_initial_fx_params.write(w);
        w.write_u8(self.override_attachment_params);
        self.initial_rtpc.write(w);
        self.state_chunk.write(w);
        if self.has_feedback { w.write_u32(self.feedback_bus_id); }
    }
}

// ─── State ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StateProp {
    pub prop_id: u8,
    pub value: f32,
}

#[derive(Debug, Clone)]
pub struct StateValues {
    pub props: Vec<StateProp>,
}

impl StateValues {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let count = r.read_u8()? as usize;
        let mut ids = Vec::with_capacity(count);
        for _ in 0..count { ids.push(r.read_u8()?); }
        let mut props = Vec::with_capacity(count);
        for i in 0..count {
            props.push(StateProp { prop_id: ids[i], value: r.read_f32()? });
        }
        Ok(Self { props })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.props.len() as u8);
        for p in &self.props { w.write_u8(p.prop_id); }
        for p in &self.props { w.write_f32(p.value); }
    }
}

// ─── Attenuation ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AttenuationValues {
    pub is_cone_enabled: bool,
    pub curve_to_use: [i8; 7],
    pub curves: Vec<ConversionTable>,
    pub initial_rtpc: InitialRtpc,
}

impl AttenuationValues {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let is_cone_enabled = r.read_u8()? == 1;
        let mut curve_to_use = [0i8; 7];
        for c in &mut curve_to_use { *c = r.read_i8()?; }
        let num_curves = r.read_u8()? as usize;
        let mut curves = Vec::with_capacity(num_curves);
        for _ in 0..num_curves { curves.push(ConversionTable::read(r)?); }
        let initial_rtpc = InitialRtpc::read(r)?;
        Ok(Self { is_cone_enabled, curve_to_use, curves, initial_rtpc })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.is_cone_enabled as u8);
        for &c in &self.curve_to_use { w.write_i8(c); }
        w.write_u8(self.curves.len() as u8);
        for c in &self.curves { c.write(w); }
        self.initial_rtpc.write(w);
    }
}

