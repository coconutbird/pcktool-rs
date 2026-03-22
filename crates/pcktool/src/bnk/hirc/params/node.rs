//! Node base parameter types.

use super::super::reader::BinaryReader;
use super::super::writer::BinaryWriter;
use super::props::PropBundle;
use super::rtpc::InitialRtpc;
use super::state::StateChunk;
use crate::error::Result;
use alloc::vec::Vec;

#[derive(Debug, Clone, Default)]
pub struct Children {
    pub child_ids: Vec<u32>,
}

impl Children {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let ids = r.read_list_u32(|r| r.read_u32())?;
        Ok(Self { child_ids: ids })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_list_u32(&self.child_ids, |w, &id| w.write_u32(id));
    }
}

#[derive(Debug, Clone)]
pub struct FxChunk {
    pub fx_index: u8,
    pub fx_id: u32,
    pub is_share_set: bool,
    pub is_rendered: bool,
}

impl FxChunk {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            fx_index: r.read_u8()?,
            fx_id: r.read_u32()?,
            is_share_set: r.read_u8()? != 0,
            is_rendered: r.read_u8()? != 0,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.fx_index);
        w.write_u32(self.fx_id);
        w.write_u8(self.is_share_set as u8);
        w.write_u8(self.is_rendered as u8);
    }
}

#[derive(Debug, Clone, Default)]
pub struct NodeInitialFxParams {
    pub is_override_parent_fx: u8,
    pub fx_bypass: bool,
    pub fx_chunks: Vec<FxChunk>,
}

impl NodeInitialFxParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let is_override_parent_fx = r.read_u8()?;
        let num_fx = r.read_u8()?;
        let (fx_bypass, fx_chunks) = if num_fx > 0 {
            let bits = r.read_u8()?;
            let bypass = (bits & 0x01) != 0;
            let mut chunks = Vec::with_capacity(num_fx as usize);
            for _ in 0..num_fx {
                chunks.push(FxChunk::read(r)?);
            }
            (bypass, chunks)
        } else {
            (false, Vec::new())
        };
        Ok(Self {
            is_override_parent_fx,
            fx_bypass,
            fx_chunks,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.is_override_parent_fx);
        w.write_u8(self.fx_chunks.len() as u8);
        if !self.fx_chunks.is_empty() {
            w.write_u8(if self.fx_bypass { 0x01 } else { 0x00 });
            for c in &self.fx_chunks {
                c.write(w);
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NodeInitialParams {
    pub prop_bundle: PropBundle,
    pub prop_bundle_ranged: PropBundle,
}

impl NodeInitialParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let prop_bundle = PropBundle::read(r, false, false)?;
        let prop_bundle_ranged = PropBundle::read(r, true, false)?;
        Ok(Self {
            prop_bundle,
            prop_bundle_ranged,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        self.prop_bundle.write(w);
        self.prop_bundle_ranged.write(w);
    }
}

#[derive(Debug, Clone)]
pub struct PathVertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub duration: i32,
}

#[derive(Debug, Clone)]
pub struct PathListItemOffset {
    pub vertices_offset: u32,
    pub num_vertices: u32,
}

#[derive(Debug, Clone)]
pub struct Ak3DAutomationParams {
    pub x_range: f32,
    pub y_range: f32,
    pub z_range: f32,
}

#[derive(Debug, Clone)]
pub struct PositioningParams {
    pub flags: u8,
    pub flags_3d: Option<u8>,
    pub attenuation_id: Option<u32>,
    pub path_mode: Option<u8>,
    pub transition_time: Option<i32>,
    pub path_vertices: Vec<PathVertex>,
    pub path_list_items: Vec<PathListItemOffset>,
    pub automation_params: Vec<Ak3DAutomationParams>,
}

impl PositioningParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let flags = r.read_u8()?;
        let is_3d = (flags & 0x08) != 0;

        let mut flags_3d = None;
        let mut attenuation_id = None;
        let mut path_mode = None;
        let mut transition_time = None;
        let mut path_vertices = Vec::new();
        let mut path_list_items = Vec::new();
        let mut automation_params = Vec::new();

        if is_3d {
            let f3d = r.read_u8()?;
            flags_3d = Some(f3d);
            attenuation_id = Some(r.read_u32()?);
            let e3d_position_type = f3d & 0x03;
            if e3d_position_type != 1 {
                path_mode = Some(r.read_u8()?);
                transition_time = Some(r.read_i32()?);
                let num_v = r.read_u32()? as usize;
                path_vertices.reserve(num_v);
                for _ in 0..num_v {
                    path_vertices.push(PathVertex {
                        x: r.read_f32()?,
                        y: r.read_f32()?,
                        z: r.read_f32()?,
                        duration: r.read_i32()?,
                    });
                }
                let num_items = r.read_u32()? as usize;
                path_list_items.reserve(num_items);
                for _ in 0..num_items {
                    path_list_items.push(PathListItemOffset {
                        vertices_offset: r.read_u32()?,
                        num_vertices: r.read_u32()?,
                    });
                }
                automation_params.reserve(num_items);
                for _ in 0..num_items {
                    automation_params.push(Ak3DAutomationParams {
                        x_range: r.read_f32()?,
                        y_range: r.read_f32()?,
                        z_range: r.read_f32()?,
                    });
                }
            }
        }

        Ok(Self {
            flags,
            flags_3d,
            attenuation_id,
            path_mode,
            transition_time,
            path_vertices,
            path_list_items,
            automation_params,
        })
    }

    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.flags);
        if let Some(f3d) = self.flags_3d {
            w.write_u8(f3d);
            w.write_u32(self.attenuation_id.unwrap_or(0));
            let e3d_position_type = f3d & 0x03;
            if e3d_position_type != 1 {
                w.write_u8(self.path_mode.unwrap_or(0));
                w.write_i32(self.transition_time.unwrap_or(0));
                w.write_u32(self.path_vertices.len() as u32);
                for v in &self.path_vertices {
                    w.write_f32(v.x);
                    w.write_f32(v.y);
                    w.write_f32(v.z);
                    w.write_i32(v.duration);
                }
                w.write_u32(self.path_list_items.len() as u32);
                for p in &self.path_list_items {
                    w.write_u32(p.vertices_offset);
                    w.write_u32(p.num_vertices);
                }
                for a in &self.automation_params {
                    w.write_f32(a.x_range);
                    w.write_f32(a.y_range);
                    w.write_f32(a.z_range);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdvSettingsParams {
    pub flags: u8,
    pub virtual_queue_behavior: u8,
    pub max_num_instances: u16,
    pub below_threshold_behavior: u8,
    pub flags2: u8,
}

impl AdvSettingsParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            flags: r.read_u8()?,
            virtual_queue_behavior: r.read_u8()?,
            max_num_instances: r.read_u16()?,
            below_threshold_behavior: r.read_u8()?,
            flags2: r.read_u8()?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.flags);
        w.write_u8(self.virtual_queue_behavior);
        w.write_u16(self.max_num_instances);
        w.write_u8(self.below_threshold_behavior);
        w.write_u8(self.flags2);
    }
}

#[derive(Debug, Clone)]
pub struct AuxParams {
    pub flags: u8,
    pub aux_ids: Option<[u32; 4]>,
}

impl AuxParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let flags = r.read_u8()?;
        let has_aux = (flags & 0x08) != 0;
        let aux_ids = if has_aux {
            Some([r.read_u32()?, r.read_u32()?, r.read_u32()?, r.read_u32()?])
        } else {
            None
        };
        Ok(Self { flags, aux_ids })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.flags);
        if let Some(ids) = &self.aux_ids {
            for &id in ids {
                w.write_u32(id);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeBaseParams {
    pub node_initial_fx_params: NodeInitialFxParams,
    pub override_attachment_params: u8,
    pub override_bus_id: u32,
    pub direct_parent_id: u32,
    pub flags: u8,
    pub node_initial_params: NodeInitialParams,
    pub positioning_params: PositioningParams,
    pub aux_params: AuxParams,
    pub adv_settings_params: AdvSettingsParams,
    pub state_chunk: StateChunk,
    pub initial_rtpc: InitialRtpc,
    pub feedback_bus_id: Option<u32>,
}

impl NodeBaseParams {
    pub fn read(r: &mut BinaryReader, has_feedback: bool) -> Result<Self> {
        let node_initial_fx_params = NodeInitialFxParams::read(r)?;
        let override_attachment_params = r.read_u8()?;
        let override_bus_id = r.read_u32()?;
        let direct_parent_id = r.read_u32()?;
        let flags = r.read_u8()?;
        let node_initial_params = NodeInitialParams::read(r)?;
        let positioning_params = PositioningParams::read(r)?;
        let aux_params = AuxParams::read(r)?;
        let adv_settings_params = AdvSettingsParams::read(r)?;
        let state_chunk = StateChunk::read(r)?;
        let initial_rtpc = InitialRtpc::read(r)?;
        let feedback_bus_id = if has_feedback {
            Some(r.read_u32()?)
        } else {
            None
        };

        Ok(Self {
            node_initial_fx_params,
            override_attachment_params,
            override_bus_id,
            direct_parent_id,
            flags,
            node_initial_params,
            positioning_params,
            aux_params,
            adv_settings_params,
            state_chunk,
            initial_rtpc,
            feedback_bus_id,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        self.node_initial_fx_params.write(w);
        w.write_u8(self.override_attachment_params);
        w.write_u32(self.override_bus_id);
        w.write_u32(self.direct_parent_id);
        w.write_u8(self.flags);
        self.node_initial_params.write(w);
        self.positioning_params.write(w);
        self.aux_params.write(w);
        self.adv_settings_params.write(w);
        self.state_chunk.write(w);
        self.initial_rtpc.write(w);
        if let Some(id) = self.feedback_bus_id {
            w.write_u32(id);
        }
    }
}
