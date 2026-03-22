//! FeedbackNode and Modulator item values.

use alloc::vec::Vec;

use crate::error::Result;
use super::super::params::*;
use super::super::reader::BinaryReader;
use super::super::writer::BinaryWriter;

// ─── FeedbackNode ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FeedbackSource {
    pub company_id: u16,
    pub device_id: u16,
    pub volume_offset: f32,
    pub bank_source_data: BankSourceData,
}

impl FeedbackSource {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            company_id: r.read_u16()?,
            device_id: r.read_u16()?,
            volume_offset: r.read_f32()?,
            bank_source_data: BankSourceData::read(r)?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u16(self.company_id);
        w.write_u16(self.device_id);
        w.write_f32(self.volume_offset);
        self.bank_source_data.write(w);
    }
}

#[derive(Debug, Clone)]
pub struct FeedbackNodeValues {
    pub sources: Vec<FeedbackSource>,
    pub node_base_params: NodeBaseParams,
}

impl FeedbackNodeValues {
    pub fn read(r: &mut BinaryReader, has_feedback: bool) -> Result<Self> {
        let sources = r.read_list_u32(FeedbackSource::read)?;
        let node_base_params = NodeBaseParams::read(r, has_feedback)?;
        Ok(Self { sources, node_base_params })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_list_u32(&self.sources, |w, s| s.write(w));
        self.node_base_params.write(w);
    }
}

// ─── Modulator ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ModulatorValues {
    pub prop_bundle: PropBundle,
    pub prop_bundle_ranged: PropBundle,
    pub initial_rtpc: InitialRtpc,
}

impl ModulatorValues {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let prop_bundle = PropBundle::read(r, false, true)?;
        let prop_bundle_ranged = PropBundle::read(r, true, true)?;
        let initial_rtpc = InitialRtpc::read(r)?;
        Ok(Self { prop_bundle, prop_bundle_ranged, initial_rtpc })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        self.prop_bundle.write(w);
        self.prop_bundle_ranged.write(w);
        self.initial_rtpc.write(w);
    }
}

