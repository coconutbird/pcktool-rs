//! Action-specific parameter types and playlist/exception types.

use super::super::reader::BinaryReader;
use super::super::writer::BinaryWriter;
use crate::error::Result;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct PlaylistItem {
    pub play_id: i32,
    pub weight: i32,
}

#[derive(Debug, Clone, Default)]
pub struct Playlist {
    pub items: Vec<PlaylistItem>,
}

impl Playlist {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let items = r.read_list_u16(|r| {
            Ok(PlaylistItem {
                play_id: r.read_i32()?,
                weight: r.read_i32()?,
            })
        })?;
        Ok(Self { items })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_list_u16(&self.items, |w, item| {
            w.write_i32(item.play_id);
            w.write_i32(item.weight);
        });
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RandomizerModifier {
    pub base: f32,
    pub min: f32,
    pub max: f32,
}

impl RandomizerModifier {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            base: r.read_f32()?,
            min: r.read_f32()?,
            max: r.read_f32()?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_f32(self.base);
        w.write_f32(self.min);
        w.write_f32(self.max);
    }
}

#[derive(Debug, Clone)]
pub struct ElementException {
    pub id: u32,
    pub is_bus_id: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ExceptParams {
    pub exceptions: Vec<ElementException>,
}

impl ExceptParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let exceptions = r.read_list_u32(|r| {
            Ok(ElementException {
                id: r.read_u32()?,
                is_bus_id: r.read_u8()? != 0,
            })
        })?;
        Ok(Self { exceptions })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_list_u32(&self.exceptions, |w, e| {
            w.write_u32(e.id);
            w.write_u8(e.is_bus_id as u8);
        });
    }
}

#[derive(Debug, Clone)]
pub struct PlayActionParams {
    pub bit_vector: u8,
    pub file_id: u32,
}

impl PlayActionParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            bit_vector: r.read_u8()?,
            file_id: r.read_u32()?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.bit_vector);
        w.write_u32(self.file_id);
    }
}

#[derive(Debug, Clone)]
pub struct ActiveActionParams {
    pub bit_vector: u8,
    pub action_specific_flags: u8,
    pub except_params: ExceptParams,
}

impl ActiveActionParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let bit_vector = r.read_u8()?;
        let action_specific_flags = r.read_u8()?;
        let except_params = ExceptParams::read(r)?;
        Ok(Self {
            bit_vector,
            action_specific_flags,
            except_params,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.bit_vector);
        w.write_u8(self.action_specific_flags);
        self.except_params.write(w);
    }
}

#[derive(Debug, Clone)]
pub struct StateActionParams {
    pub state_group_id: u32,
    pub target_state_id: u32,
}

impl StateActionParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            state_group_id: r.read_u32()?,
            target_state_id: r.read_u32()?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u32(self.state_group_id);
        w.write_u32(self.target_state_id);
    }
}

#[derive(Debug, Clone)]
pub struct SwitchActionParams {
    pub switch_group_id: u32,
    pub switch_state_id: u32,
}

impl SwitchActionParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            switch_group_id: r.read_u32()?,
            switch_state_id: r.read_u32()?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u32(self.switch_group_id);
        w.write_u32(self.switch_state_id);
    }
}

#[derive(Debug, Clone)]
pub struct GameParamActionParams {
    pub bit_vector: u8,
    pub bypass_transition: u8,
    pub value_meaning: u8,
    pub base: f32,
    pub min: f32,
    pub max: f32,
    pub except_params: ExceptParams,
}

impl GameParamActionParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            bit_vector: r.read_u8()?,
            bypass_transition: r.read_u8()?,
            value_meaning: r.read_u8()?,
            base: r.read_f32()?,
            min: r.read_f32()?,
            max: r.read_f32()?,
            except_params: ExceptParams::read(r)?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.bit_vector);
        w.write_u8(self.bypass_transition);
        w.write_u8(self.value_meaning);
        w.write_f32(self.base);
        w.write_f32(self.min);
        w.write_f32(self.max);
        self.except_params.write(w);
    }
}

#[derive(Debug, Clone)]
pub struct ValueActionParams {
    pub bit_vector: u8,
    pub value_meaning: u8,
    pub randomizer: RandomizerModifier,
    pub except_params: ExceptParams,
}

impl ValueActionParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            bit_vector: r.read_u8()?,
            value_meaning: r.read_u8()?,
            randomizer: RandomizerModifier::read(r)?,
            except_params: ExceptParams::read(r)?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.bit_vector);
        w.write_u8(self.value_meaning);
        self.randomizer.write(w);
        self.except_params.write(w);
    }
}

#[derive(Debug, Clone)]
pub struct BypassFXActionParams {
    pub is_bypass: u8,
    pub target_mask: u8,
    pub except_params: ExceptParams,
}

impl BypassFXActionParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            is_bypass: r.read_u8()?,
            target_mask: r.read_u8()?,
            except_params: ExceptParams::read(r)?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.is_bypass);
        w.write_u8(self.target_mask);
        self.except_params.write(w);
    }
}

#[derive(Debug, Clone)]
pub struct SeekActionParams {
    pub is_seek_relative: u8,
    pub seek_value: RandomizerModifier,
    pub snap_to_nearest_marker: u8,
    pub except_params: ExceptParams,
}

impl SeekActionParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            is_seek_relative: r.read_u8()?,
            seek_value: RandomizerModifier::read(r)?,
            snap_to_nearest_marker: r.read_u8()?,
            except_params: ExceptParams::read(r)?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.is_seek_relative);
        self.seek_value.write(w);
        w.write_u8(self.snap_to_nearest_marker);
        self.except_params.write(w);
    }
}

#[derive(Debug, Clone)]
pub struct PropActionSpecificParams {
    pub value_meaning: u8,
    pub randomizer_modifier: RandomizerModifier,
}

impl PropActionSpecificParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            value_meaning: r.read_u8()?,
            randomizer_modifier: RandomizerModifier::read(r)?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.value_meaning);
        self.randomizer_modifier.write(w);
    }
}

#[derive(Debug, Clone)]
pub struct ResumeActionSpecificParams {
    pub flags: u8,
}

impl ResumeActionSpecificParams {
    pub const FLAG_IS_MASTER_RESUME: u8 = 0x01;
    pub const FLAG_APPLY_TO_STATE_TRANSITIONS: u8 = 0x02;
    pub const FLAG_APPLY_TO_DYNAMIC_SEQUENCE: u8 = 0x04;

    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            flags: r.read_u8()?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.flags);
    }
}
