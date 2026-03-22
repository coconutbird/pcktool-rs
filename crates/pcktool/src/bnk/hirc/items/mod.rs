//! HIRC item types — one variant per Wwise object kind.

pub mod sound;
pub mod action;
pub mod containers;
pub mod bus;
pub mod fx;
pub mod misc;
pub mod music;

pub use sound::*;
pub use action::*;
pub use containers::*;
pub use bus::*;
pub use fx::*;
pub use misc::*;
pub use music::*;

use alloc::vec::Vec;

use crate::error::Result;
use super::reader::BinaryReader;
use super::writer::BinaryWriter;
use super::types::HircType;

// ─── HircItem ───────────────────────────────────────────────────────

/// A parsed HIRC item (type + ID + data).
#[derive(Debug, Clone)]
pub struct HircItem {
    pub hirc_type: HircType,
    pub id: u32,
    pub body: HircBody,
}

/// The item-specific payload.
#[derive(Debug, Clone)]
pub enum HircBody {
    Sound(SoundValues),
    Action(ActionValues),
    Event(EventValues),
    RanSeqCntr(RanSeqCntrValues),
    SwitchCntr(SwitchCntrValues),
    ActorMixer(ActorMixerValues),
    Bus(BusValues),
    LayerCntr(LayerCntrValues),
    State(StateValues),
    Attenuation(AttenuationValues),
    DialogueEvent(DialogueEventValues),
    Fx(FxBaseValues),
    MusicSegment(MusicSegmentValues),
    MusicTrack(MusicTrackValues),
    MusicSwitch(MusicSwitchValues),
    MusicRanSeq(MusicRanSeqValues),
    FeedbackNode(FeedbackNodeValues),
    Modulator(ModulatorValues),
    /// Fallback for unknown/unimplemented types.
    Unknown(Vec<u8>),
}

// ─── Top-level read/write ───────────────────────────────────────────

impl HircItem {
    /// Read a single HIRC item (type + size + id + body).
    pub fn read(r: &mut BinaryReader, has_feedback: bool) -> Result<Self> {
        let type_byte = r.read_u8()?;
        let section_size = r.read_u32()? as usize;
        let start_pos = r.position();
        let id = r.read_u32()?;
        let remaining = section_size.saturating_sub(4);

        let hirc_type_opt = HircType::from_u8(type_byte);

        let body = match hirc_type_opt {
            Some(HircType::Sound) => HircBody::Sound(SoundValues::read(r, has_feedback)?),
            Some(HircType::Action) => HircBody::Action(ActionValues::read(r)?),
            Some(HircType::Event) => HircBody::Event(EventValues::read(r)?),
            Some(HircType::RanSeqCntr) => HircBody::RanSeqCntr(RanSeqCntrValues::read(r, has_feedback)?),
            Some(HircType::SwitchCntr) => HircBody::SwitchCntr(SwitchCntrValues::read(r, has_feedback)?),
            Some(HircType::ActorMixer) => HircBody::ActorMixer(ActorMixerValues::read(r, has_feedback)?),
            Some(HircType::Bus) | Some(HircType::FeedbackBus) | Some(HircType::AuxBus) =>
                HircBody::Bus(BusValues::read(r, has_feedback)?),
            Some(HircType::LayerCntr) => HircBody::LayerCntr(LayerCntrValues::read(r, has_feedback)?),
            Some(HircType::State) => HircBody::State(StateValues::read(r)?),
            Some(HircType::Attenuation) => HircBody::Attenuation(AttenuationValues::read(r)?),
            Some(HircType::DialogueEvent) => HircBody::DialogueEvent(DialogueEventValues::read(r)?),
            Some(HircType::FxShareSet) | Some(HircType::FxCustom) | Some(HircType::AudioDevice) =>
                HircBody::Fx(FxBaseValues::read(r)?),
            Some(HircType::MusicSegment) => HircBody::MusicSegment(MusicSegmentValues::read(r, has_feedback)?),
            Some(HircType::MusicTrack) => HircBody::MusicTrack(MusicTrackValues::read(r, has_feedback)?),
            Some(HircType::MusicSwitch) => HircBody::MusicSwitch(MusicSwitchValues::read(r, has_feedback)?),
            Some(HircType::MusicRanSeq) => HircBody::MusicRanSeq(MusicRanSeqValues::read(r, has_feedback)?),
            Some(HircType::FeedbackNode) => HircBody::FeedbackNode(FeedbackNodeValues::read(r, has_feedback)?),
            Some(HircType::LfoModulator) | Some(HircType::EnvelopeModulator) =>
                HircBody::Modulator(ModulatorValues::read(r)?),
            _ => {
                let data = r.read_bytes(remaining)?.to_vec();
                HircBody::Unknown(data)
            }
        };

        let hirc_type = hirc_type_opt.unwrap_or(HircType::State); // fallback for unknown

        // Seek to expected end position (skip any unread bytes)
        let expected_end = start_pos + section_size;
        if r.position() < expected_end {
            let skip = expected_end - r.position();
            let _ = r.read_bytes(skip);
        }

        Ok(HircItem { hirc_type, id, body })
    }

    /// Write this HIRC item (type + size + id + body).
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.hirc_type as u8);
        // Reserve space for size
        let size_pos = w.position();
        w.write_u32(0);
        let content_start = w.position();
        w.write_u32(self.id);

        match &self.body {
            HircBody::Sound(v) => v.write(w),
            HircBody::Action(v) => v.write(w),
            HircBody::Event(v) => v.write(w),
            HircBody::RanSeqCntr(v) => v.write(w),
            HircBody::SwitchCntr(v) => v.write(w),
            HircBody::ActorMixer(v) => v.write(w),
            HircBody::Bus(v) => v.write(w),
            HircBody::LayerCntr(v) => v.write(w),
            HircBody::State(v) => v.write(w),
            HircBody::Attenuation(v) => v.write(w),
            HircBody::DialogueEvent(v) => v.write(w),
            HircBody::Fx(v) => v.write(w),
            HircBody::MusicSegment(v) => v.write(w),
            HircBody::MusicTrack(v) => v.write(w),
            HircBody::MusicSwitch(v) => v.write(w),
            HircBody::MusicRanSeq(v) => v.write(w),
            HircBody::FeedbackNode(v) => v.write(w),
            HircBody::Modulator(v) => v.write(w),
            HircBody::Unknown(data) => w.write_bytes(data),
        }

        let content_end = w.position();
        let size = (content_end - content_start) as u32;
        w.patch_u32(size_pos, size);
    }
}

