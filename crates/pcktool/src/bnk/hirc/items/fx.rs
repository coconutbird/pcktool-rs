//! DialogueEvent and FxBase item values.

use alloc::vec::Vec;

use super::super::params::*;
use super::super::reader::BinaryReader;
use super::super::writer::BinaryWriter;
use crate::error::Result;

// ─── DialogueEvent ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GameSyncArgument {
    pub group_id: u32,
    pub group_type: u8,
}

#[derive(Debug, Clone)]
pub struct DialogueEventValues {
    pub probability: u8,
    pub tree_depth: u32,
    pub arguments: Vec<GameSyncArgument>,
    pub tree_data_size: u32,
    pub tree_mode: u8,
    pub decision_tree: DecisionTree,
}

impl DialogueEventValues {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let probability = r.read_u8()?;
        let tree_depth = r.read_u32()?;
        let mut group_ids = Vec::with_capacity(tree_depth as usize);
        let mut group_types = Vec::with_capacity(tree_depth as usize);
        for _ in 0..tree_depth {
            group_ids.push(r.read_u32()?);
        }
        for _ in 0..tree_depth {
            group_types.push(r.read_u8()?);
        }
        let arguments: Vec<GameSyncArgument> = group_ids
            .into_iter()
            .zip(group_types)
            .map(|(id, ty)| GameSyncArgument {
                group_id: id,
                group_type: ty,
            })
            .collect();
        let tree_data_size = r.read_u32()?;
        let tree_mode = r.read_u8()?;
        let decision_tree = DecisionTree::read(r, tree_depth, tree_data_size)?;
        Ok(Self {
            probability,
            tree_depth,
            arguments,
            tree_data_size,
            tree_mode,
            decision_tree,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.probability);
        w.write_u32(self.tree_depth);
        for a in &self.arguments {
            w.write_u32(a.group_id);
        }
        for a in &self.arguments {
            w.write_u8(a.group_type);
        }
        w.write_u32(self.tree_data_size);
        w.write_u8(self.tree_mode);
        self.decision_tree.write(w, self.tree_depth);
    }
}

// ─── FxBase ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MediaMapEntry {
    pub index: u8,
    pub source_id: u32,
}

#[derive(Debug, Clone)]
pub struct RtpcInit {
    pub param_id: u8,
    pub init_value: f32,
}

#[derive(Debug, Clone)]
pub struct FxBaseValues {
    pub fx_id: PluginId,
    pub size: u32,
    pub plugin_param: Option<PluginParam>,
    pub fx_src_silence_params: Option<FxSrcSilenceParams>,
    pub delay_fx_params: Option<DelayFxParams>,
    pub raw_plugin_data: Option<Vec<u8>>,
    pub media_map: Vec<MediaMapEntry>,
    pub initial_rtpc: InitialRtpc,
    pub rtpc_init_list: Vec<RtpcInit>,
}

impl FxBaseValues {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let fx_id = PluginId(r.read_u32()?);
        let size = r.read_u32()?;
        let mut plugin_param = None;
        let mut fx_src_silence_params = None;
        let mut delay_fx_params = None;
        let mut raw_plugin_data = None;
        if size > 0 {
            match fx_id {
                PluginId::WWISE_COMPRESSOR => plugin_param = Some(PluginParam::read(r, size)?),
                PluginId::WWISE_SILENCE => {
                    fx_src_silence_params = Some(FxSrcSilenceParams::read(r)?)
                }
                PluginId::WWISE_DELAY => delay_fx_params = Some(DelayFxParams::read(r)?),
                _ => raw_plugin_data = Some(r.read_bytes(size as usize)?.to_vec()),
            }
        }
        let num_bank_data = r.read_u8()? as usize;
        let mut media_map = Vec::with_capacity(num_bank_data);
        for _ in 0..num_bank_data {
            media_map.push(MediaMapEntry {
                index: r.read_u8()?,
                source_id: r.read_u32()?,
            });
        }
        let initial_rtpc = InitialRtpc::read(r)?;
        let num_init = r.read_u16()? as usize;
        let mut rtpc_init_list = Vec::with_capacity(num_init);
        for _ in 0..num_init {
            rtpc_init_list.push(RtpcInit {
                param_id: r.read_u8()?,
                init_value: r.read_f32()?,
            });
        }
        Ok(Self {
            fx_id,
            size,
            plugin_param,
            fx_src_silence_params,
            delay_fx_params,
            raw_plugin_data,
            media_map,
            initial_rtpc,
            rtpc_init_list,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u32(self.fx_id.0);
        w.write_u32(self.size);
        if self.size > 0 {
            match self.fx_id {
                PluginId::WWISE_COMPRESSOR => {
                    if let Some(p) = &self.plugin_param {
                        p.write(w);
                    }
                }
                PluginId::WWISE_SILENCE => {
                    if let Some(p) = &self.fx_src_silence_params {
                        p.write(w);
                    }
                }
                PluginId::WWISE_DELAY => {
                    if let Some(p) = &self.delay_fx_params {
                        p.write(w);
                    }
                }
                _ => {
                    if let Some(d) = &self.raw_plugin_data {
                        w.write_bytes(d);
                    }
                }
            }
        }
        w.write_u8(self.media_map.len() as u8);
        for e in &self.media_map {
            w.write_u8(e.index);
            w.write_u32(e.source_id);
        }
        self.initial_rtpc.write(w);
        w.write_u16(self.rtpc_init_list.len() as u16);
        for i in &self.rtpc_init_list {
            w.write_u8(i.param_id);
            w.write_f32(i.init_value);
        }
    }
}
