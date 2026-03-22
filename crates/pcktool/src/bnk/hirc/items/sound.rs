//! Sound, Event, and ActorMixer item values.

use alloc::vec::Vec;

use super::super::params::*;
use super::super::reader::BinaryReader;
use super::super::writer::BinaryWriter;
use crate::error::Result;

// ─── Sound ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SoundValues {
    pub bank_source_data: BankSourceData,
    pub node_base_params: NodeBaseParams,
}

impl SoundValues {
    pub fn read(r: &mut BinaryReader, has_feedback: bool) -> Result<Self> {
        Ok(Self {
            bank_source_data: BankSourceData::read(r)?,
            node_base_params: NodeBaseParams::read(r, has_feedback)?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        self.bank_source_data.write(w);
        self.node_base_params.write(w);
    }
}

// ─── Event ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EventValues {
    pub actions: Vec<u32>,
}

impl EventValues {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let actions = r.read_list_u32(|r| r.read_u32())?;
        Ok(Self { actions })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_list_u32(&self.actions, |w, &v| w.write_u32(v));
    }
}

// ─── ActorMixer ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ActorMixerValues {
    pub node_base_params: NodeBaseParams,
    pub children: Children,
}

impl ActorMixerValues {
    pub fn read(r: &mut BinaryReader, has_feedback: bool) -> Result<Self> {
        Ok(Self {
            node_base_params: NodeBaseParams::read(r, has_feedback)?,
            children: Children::read(r)?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        self.node_base_params.write(w);
        self.children.write(w);
    }
}
