//! Property types and bundles.

use super::super::reader::BinaryReader;
use super::super::writer::BinaryWriter;
use crate::error::Result;
use alloc::vec::Vec;

/// A single property: ID + raw value bytes.
#[derive(Debug, Clone)]
pub struct Prop {
    pub id: u8,
    pub raw_value: Vec<u8>,
}

/// Determines byte size of a property value by ID.
fn prop_value_size(_id: u8, _is_ranged: bool, _is_modulator: bool) -> usize {
    4
}

/// A collection of properties, read in two passes (IDs then values).
#[derive(Debug, Clone, Default)]
pub struct PropBundle {
    pub props: Vec<Prop>,
}

impl PropBundle {
    pub fn read(r: &mut BinaryReader, is_ranged: bool, is_modulator: bool) -> Result<Self> {
        let count = r.read_u8()? as usize;
        let mut ids = Vec::with_capacity(count);
        for _ in 0..count {
            ids.push(r.read_u8()?);
        }
        let mut props = Vec::with_capacity(count);
        for &id in &ids {
            let size = prop_value_size(id, is_ranged, is_modulator);
            let raw = r.read_bytes(size)?.to_vec();
            props.push(Prop { id, raw_value: raw });
        }
        Ok(Self { props })
    }

    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.props.len() as u8);
        for p in &self.props {
            w.write_u8(p.id);
        }
        for p in &self.props {
            w.write_bytes(&p.raw_value);
        }
    }
}
