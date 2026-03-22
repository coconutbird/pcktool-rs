//! State chunk types.

use super::super::reader::BinaryReader;
use super::super::writer::BinaryWriter;
use crate::error::Result;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct StateEntry {
    pub state_id: u32,
    pub state_instance_id: u32,
}

#[derive(Debug, Clone)]
pub struct StateGroup {
    pub state_group_id: u32,
    pub state_sync_type: u8,
    pub states: Vec<StateEntry>,
}

#[derive(Debug, Clone, Default)]
pub struct StateChunk {
    pub state_groups: Vec<StateGroup>,
}

impl StateChunk {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let num_groups = r.read_u32()? as usize;
        let mut state_groups = Vec::with_capacity(num_groups);
        for _ in 0..num_groups {
            let state_group_id = r.read_u32()?;
            let state_sync_type = r.read_u8()?;
            let num_states = r.read_u16()? as usize;
            let mut states = Vec::with_capacity(num_states);
            for _ in 0..num_states {
                states.push(StateEntry {
                    state_id: r.read_u32()?,
                    state_instance_id: r.read_u32()?,
                });
            }
            state_groups.push(StateGroup {
                state_group_id,
                state_sync_type,
                states,
            });
        }
        Ok(Self { state_groups })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u32(self.state_groups.len() as u32);
        for g in &self.state_groups {
            w.write_u32(g.state_group_id);
            w.write_u8(g.state_sync_type);
            w.write_u16(g.states.len() as u16);
            for s in &g.states {
                w.write_u32(s.state_id);
                w.write_u32(s.state_instance_id);
            }
        }
    }
}
