//! Container item values: RanSeqCntr, SwitchCntr, LayerCntr.

use alloc::vec::Vec;

use crate::error::Result;
use super::super::params::*;
use super::super::reader::BinaryReader;
use super::super::writer::BinaryWriter;

// ─── RanSeqCntr ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RanSeqCntrValues {
    pub node_base_params: NodeBaseParams,
    pub loop_count: u16,
    pub loop_mod_min: u16,
    pub loop_mod_max: u16,
    pub transition_time: f32,
    pub transition_time_mod_min: f32,
    pub transition_time_mod_max: f32,
    pub avoid_repeat_count: u16,
    pub transition_mode: u8,
    pub random_mode: u8,
    pub mode: u8,
    pub bit_vector: u8,
    pub children: Children,
    pub playlist: Playlist,
}

impl RanSeqCntrValues {
    pub fn read(r: &mut BinaryReader, has_feedback: bool) -> Result<Self> {
        let node_base_params = NodeBaseParams::read(r, has_feedback)?;
        let loop_count = r.read_u16()?;
        let loop_mod_min = r.read_u16()?;
        let loop_mod_max = r.read_u16()?;
        let transition_time = r.read_f32()?;
        let transition_time_mod_min = r.read_f32()?;
        let transition_time_mod_max = r.read_f32()?;
        let avoid_repeat_count = r.read_u16()?;
        let transition_mode = r.read_u8()?;
        let random_mode = r.read_u8()?;
        let mode = r.read_u8()?;
        let bit_vector = r.read_u8()?;
        let children = Children::read(r)?;
        let playlist = Playlist::read(r)?;
        Ok(Self {
            node_base_params, loop_count, loop_mod_min, loop_mod_max,
            transition_time, transition_time_mod_min, transition_time_mod_max,
            avoid_repeat_count, transition_mode, random_mode, mode, bit_vector,
            children, playlist,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        self.node_base_params.write(w);
        w.write_u16(self.loop_count);
        w.write_u16(self.loop_mod_min);
        w.write_u16(self.loop_mod_max);
        w.write_f32(self.transition_time);
        w.write_f32(self.transition_time_mod_min);
        w.write_f32(self.transition_time_mod_max);
        w.write_u16(self.avoid_repeat_count);
        w.write_u8(self.transition_mode);
        w.write_u8(self.random_mode);
        w.write_u8(self.mode);
        w.write_u8(self.bit_vector);
        self.children.write(w);
        self.playlist.write(w);
    }
}

// ─── SwitchCntr ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SwitchPackage {
    pub switch_id: u32,
    pub node_ids: Vec<u32>,
}

impl SwitchPackage {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let switch_id = r.read_u32()?;
        let node_ids = r.read_list_u32(|r| r.read_u32())?;
        Ok(Self { switch_id, node_ids })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u32(self.switch_id);
        w.write_list_u32(&self.node_ids, |w, &v| w.write_u32(v));
    }
}

#[derive(Debug, Clone)]
pub struct SwitchNodeParams {
    pub node_id: u32,
    pub bit_vector: u8,
    pub mode_bit_vector: u8,
    pub fade_out_time: i32,
    pub fade_in_time: i32,
}

impl SwitchNodeParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            node_id: r.read_u32()?,
            bit_vector: r.read_u8()?,
            mode_bit_vector: r.read_u8()?,
            fade_out_time: r.read_i32()?,
            fade_in_time: r.read_i32()?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u32(self.node_id);
        w.write_u8(self.bit_vector);
        w.write_u8(self.mode_bit_vector);
        w.write_i32(self.fade_out_time);
        w.write_i32(self.fade_in_time);
    }
}

#[derive(Debug, Clone)]
pub struct SwitchCntrValues {
    pub node_base_params: NodeBaseParams,
    pub group_type: u8,
    pub group_id: u32,
    pub default_switch: u32,
    pub is_continuous_validation: u8,
    pub children: Children,
    pub switch_list: Vec<SwitchPackage>,
    pub switch_params: Vec<SwitchNodeParams>,
}

impl SwitchCntrValues {
    pub fn read(r: &mut BinaryReader, has_feedback: bool) -> Result<Self> {
        let node_base_params = NodeBaseParams::read(r, has_feedback)?;
        let group_type = r.read_u8()?;
        let group_id = r.read_u32()?;
        let default_switch = r.read_u32()?;
        let is_continuous_validation = r.read_u8()?;
        let children = Children::read(r)?;
        let switch_list = r.read_list_u32(SwitchPackage::read)?;
        let switch_params = r.read_list_u32(SwitchNodeParams::read)?;
        Ok(Self {
            node_base_params, group_type, group_id, default_switch,
            is_continuous_validation, children, switch_list, switch_params,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        self.node_base_params.write(w);
        w.write_u8(self.group_type);
        w.write_u32(self.group_id);
        w.write_u32(self.default_switch);
        w.write_u8(self.is_continuous_validation);
        self.children.write(w);
        w.write_list_u32(&self.switch_list, |w, p| p.write(w));
        w.write_list_u32(&self.switch_params, |w, p| p.write(w));
    }
}

// ─── LayerCntr ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AssociatedChildData {
    pub associated_child_id: u32,
    pub curve: Vec<RtpcGraphPoint>,
}

impl AssociatedChildData {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let associated_child_id = r.read_u32()?;
        let curve = r.read_list_u32(RtpcGraphPoint::read)?;
        Ok(Self { associated_child_id, curve })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u32(self.associated_child_id);
        w.write_list_u32(&self.curve, |w, p| p.write(w));
    }
}

#[derive(Debug, Clone)]
pub struct LayerValues {
    pub layer_id: u32,
    pub initial_rtpc: InitialRtpc,
    pub rtpc_id: u32,
    pub rtpc_type: u8,
    pub associations: Vec<AssociatedChildData>,
}

impl LayerValues {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let layer_id = r.read_u32()?;
        let initial_rtpc = InitialRtpc::read(r)?;
        let rtpc_id = r.read_u32()?;
        let rtpc_type = r.read_u8()?;
        let associations = r.read_list_u32(AssociatedChildData::read)?;
        Ok(Self { layer_id, initial_rtpc, rtpc_id, rtpc_type, associations })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u32(self.layer_id);
        self.initial_rtpc.write(w);
        w.write_u32(self.rtpc_id);
        w.write_u8(self.rtpc_type);
        w.write_list_u32(&self.associations, |w, a| a.write(w));
    }
}

#[derive(Debug, Clone)]
pub struct LayerCntrValues {
    pub node_base_params: NodeBaseParams,
    pub children: Children,
    pub layers: Vec<LayerValues>,
}

impl LayerCntrValues {
    pub fn read(r: &mut BinaryReader, has_feedback: bool) -> Result<Self> {
        let node_base_params = NodeBaseParams::read(r, has_feedback)?;
        let children = Children::read(r)?;
        let layers = r.read_list_u32(LayerValues::read)?;
        Ok(Self { node_base_params, children, layers })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        self.node_base_params.write(w);
        self.children.write(w);
        w.write_list_u32(&self.layers, |w, l| l.write(w));
    }
}

