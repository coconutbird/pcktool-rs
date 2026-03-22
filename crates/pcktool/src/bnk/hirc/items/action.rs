//! Action item values.

use super::super::params::*;
use super::super::reader::BinaryReader;
use super::super::types::{ActionCategory, ActionType};
use super::super::writer::BinaryWriter;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct ActionValues {
    pub action_type: ActionType,
    pub ext: u32,
    pub ext4: u8,
    pub prop_bundle1: PropBundle,
    pub prop_bundle2: PropBundle,
    pub play_params: Option<PlayActionParams>,
    pub active_params: Option<ActiveActionParams>,
    pub state_params: Option<StateActionParams>,
    pub switch_params: Option<SwitchActionParams>,
    pub game_param_params: Option<GameParamActionParams>,
    pub value_params: Option<ValueActionParams>,
    pub bypass_fx_params: Option<BypassFXActionParams>,
    pub seek_params: Option<SeekActionParams>,
}

impl ActionValues {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let action_type = ActionType(r.read_u16()?);
        let ext = r.read_u32()?;
        let ext4 = r.read_u8()?;
        let prop_bundle1 = PropBundle::read(r, false, false)?;
        let prop_bundle2 = PropBundle::read(r, true, false)?;

        let cat = action_type.category();
        let mut v = ActionValues {
            action_type,
            ext,
            ext4,
            prop_bundle1,
            prop_bundle2,
            play_params: None,
            active_params: None,
            state_params: None,
            switch_params: None,
            game_param_params: None,
            value_params: None,
            bypass_fx_params: None,
            seek_params: None,
        };

        match cat {
            ActionCategory::Play => v.play_params = Some(PlayActionParams::read(r)?),
            ActionCategory::Active => v.active_params = Some(ActiveActionParams::read(r)?),
            ActionCategory::State => v.state_params = Some(StateActionParams::read(r)?),
            ActionCategory::Switch => v.switch_params = Some(SwitchActionParams::read(r)?),
            ActionCategory::GameParam => {
                v.game_param_params = Some(GameParamActionParams::read(r)?)
            }
            ActionCategory::Value => v.value_params = Some(ValueActionParams::read(r)?),
            ActionCategory::BypassFX => v.bypass_fx_params = Some(BypassFXActionParams::read(r)?),
            ActionCategory::Seek => v.seek_params = Some(SeekActionParams::read(r)?),
            ActionCategory::None | ActionCategory::Event | ActionCategory::PlayEvent => {}
            ActionCategory::Unknown => {}
        }
        Ok(v)
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u16(self.action_type.0);
        w.write_u32(self.ext);
        w.write_u8(self.ext4);
        self.prop_bundle1.write(w);
        self.prop_bundle2.write(w);

        let cat = self.action_type.category();
        match cat {
            ActionCategory::Play => {
                if let Some(p) = &self.play_params {
                    p.write(w);
                }
            }
            ActionCategory::Active => {
                if let Some(p) = &self.active_params {
                    p.write(w);
                }
            }
            ActionCategory::State => {
                if let Some(p) = &self.state_params {
                    p.write(w);
                }
            }
            ActionCategory::Switch => {
                if let Some(p) = &self.switch_params {
                    p.write(w);
                }
            }
            ActionCategory::GameParam => {
                if let Some(p) = &self.game_param_params {
                    p.write(w);
                }
            }
            ActionCategory::Value => {
                if let Some(p) = &self.value_params {
                    p.write(w);
                }
            }
            ActionCategory::BypassFX => {
                if let Some(p) = &self.bypass_fx_params {
                    p.write(w);
                }
            }
            ActionCategory::Seek => {
                if let Some(p) = &self.seek_params {
                    p.write(w);
                }
            }
            _ => {}
        }
    }
}
