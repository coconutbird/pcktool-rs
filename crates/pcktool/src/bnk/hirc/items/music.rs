//! Music item values: MusicSegment, MusicTrack, MusicSwitch, MusicRanSeq.

use alloc::string::String;
use alloc::vec::Vec;

use crate::error::Result;
use super::super::params::*;
use super::super::reader::BinaryReader;
use super::super::writer::BinaryWriter;
use super::fx::GameSyncArgument;

// ─── MusicSegment ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MusicMarker {
    pub id: u32,
    pub position: f64,
    pub name: Option<String>,
}

impl MusicMarker {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let id = r.read_u32()?;
        let position = r.read_f64()?;
        let string_size = r.read_u32()? as usize;
        let name = if string_size > 0 {
            let s = r.read_string(string_size)?;
            Some(s)
        } else { None };
        Ok(Self { id, position, name })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u32(self.id);
        w.write_f64(self.position);
        if let Some(n) = &self.name {
            let bytes_len = n.len() + 1;
            w.write_u32(bytes_len as u32);
            w.write_string(n);
            w.write_u8(0);
        } else {
            w.write_u32(0);
        }
    }
}

#[derive(Debug, Clone)]
pub struct MusicSegmentValues {
    pub music_node_params: MusicNodeParams,
    pub duration: f64,
    pub markers: Vec<MusicMarker>,
}

impl MusicSegmentValues {
    pub fn read(r: &mut BinaryReader, has_feedback: bool) -> Result<Self> {
        let music_node_params = MusicNodeParams::read(r, has_feedback)?;
        let duration = r.read_f64()?;
        let markers = r.read_list_u32(MusicMarker::read)?;
        Ok(Self { music_node_params, duration, markers })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        self.music_node_params.write(w);
        w.write_f64(self.duration);
        w.write_list_u32(&self.markers, |w, m| m.write(w));
    }
}

// ─── MusicTrack ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MusicTrackValues {
    pub overrides: u8,
    pub sources: Vec<BankSourceData>,
    pub playlist: Vec<TrackSrcInfo>,
    pub num_sub_track: u32,
    pub clip_automation_items: Vec<ClipAutomation>,
    pub node_base_params: NodeBaseParams,
    pub track_type: u8,
    pub switch_params: Option<TrackSwitchParams>,
    pub trans_params: Option<TrackTransParams>,
    pub look_ahead_time: i32,
}

impl MusicTrackValues {
    pub fn read(r: &mut BinaryReader, has_feedback: bool) -> Result<Self> {
        let overrides = r.read_u8()?;
        let num_sources = r.read_u32()? as usize;
        let mut sources = Vec::with_capacity(num_sources);
        for _ in 0..num_sources { sources.push(BankSourceData::read(r)?); }
        let num_playlist = r.read_u32()? as usize;
        let mut playlist = Vec::with_capacity(num_playlist);
        let mut num_sub_track = 0u32;
        if num_playlist > 0 {
            for _ in 0..num_playlist { playlist.push(TrackSrcInfo::read(r)?); }
            num_sub_track = r.read_u32()?;
        }
        let clip_automation_items = r.read_list_u32(ClipAutomation::read)?;
        let node_base_params = NodeBaseParams::read(r, has_feedback)?;
        let track_type = r.read_u8()?;
        let (switch_params, trans_params) = if track_type == 0x3 {
            (Some(TrackSwitchParams::read(r)?), Some(TrackTransParams::read(r)?))
        } else { (None, None) };
        let look_ahead_time = r.read_i32()?;
        Ok(Self { overrides, sources, playlist, num_sub_track, clip_automation_items, node_base_params, track_type, switch_params, trans_params, look_ahead_time })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.overrides);
        w.write_u32(self.sources.len() as u32);
        for s in &self.sources { s.write(w); }
        w.write_u32(self.playlist.len() as u32);
        if !self.playlist.is_empty() {
            for p in &self.playlist { p.write(w); }
            w.write_u32(self.num_sub_track);
        }
        w.write_list_u32(&self.clip_automation_items, |w, c| c.write(w));
        self.node_base_params.write(w);
        w.write_u8(self.track_type);
        if self.track_type == 0x3 {
            if let Some(sp) = &self.switch_params { sp.write(w); }
            if let Some(tp) = &self.trans_params { tp.write(w); }
        }
        w.write_i32(self.look_ahead_time);
    }
}

// ─── MusicSwitch ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MusicSwitchValues {
    pub music_trans_node_params: MusicTransNodeParams,
    pub is_continue_playback: u8,
    pub tree_depth: u32,
    pub arguments: Vec<GameSyncArgument>,
    pub tree_data_size: u32,
    pub tree_mode: u8,
    pub decision_tree: DecisionTree,
}

impl MusicSwitchValues {
    pub fn read(r: &mut BinaryReader, has_feedback: bool) -> Result<Self> {
        let music_trans_node_params = MusicTransNodeParams::read(r, has_feedback)?;
        let is_continue_playback = r.read_u8()?;
        let tree_depth = r.read_u32()?;
        let mut group_ids = Vec::with_capacity(tree_depth as usize);
        let mut group_types = Vec::with_capacity(tree_depth as usize);
        for _ in 0..tree_depth { group_ids.push(r.read_u32()?); }
        for _ in 0..tree_depth { group_types.push(r.read_u8()?); }
        let arguments: Vec<GameSyncArgument> = group_ids.into_iter().zip(group_types)
            .map(|(id, ty)| GameSyncArgument { group_id: id, group_type: ty })
            .collect();
        let tree_data_size = r.read_u32()?;
        let tree_mode = r.read_u8()?;
        let decision_tree = DecisionTree::read(r, tree_depth, tree_data_size)?;
        Ok(Self { music_trans_node_params, is_continue_playback, tree_depth, arguments, tree_data_size, tree_mode, decision_tree })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        self.music_trans_node_params.write(w);
        w.write_u8(self.is_continue_playback);
        w.write_u32(self.tree_depth);
        for a in &self.arguments { w.write_u32(a.group_id); }
        for a in &self.arguments { w.write_u8(a.group_type); }
        w.write_u32(self.tree_data_size);
        w.write_u8(self.tree_mode);
        self.decision_tree.write(w, self.tree_depth);
    }
}

// ─── MusicRanSeq ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MusicRanSeqPlaylistItem {
    pub segment_id: u32,
    pub playlist_item_id: u32,
    pub num_children: u32,
    pub rs_type: u32,
    pub loop_val: i16,
    pub loop_min: i16,
    pub loop_max: i16,
    pub weight: u32,
    pub avoid_repeat_count: u16,
    pub is_using_weight: u8,
    pub is_shuffle: u8,
    pub children: Vec<MusicRanSeqPlaylistItem>,
}

impl MusicRanSeqPlaylistItem {
    fn read_recursive(r: &mut BinaryReader, count: u32) -> Result<Vec<Self>> {
        let mut items = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let segment_id = r.read_u32()?;
            let playlist_item_id = r.read_u32()?;
            let num_children = r.read_u32()?;
            let rs_type = r.read_u32()?;
            let loop_val = r.read_i16()?;
            let loop_min = r.read_i16()?;
            let loop_max = r.read_i16()?;
            let weight = r.read_u32()?;
            let avoid_repeat_count = r.read_u16()?;
            let is_using_weight = r.read_u8()?;
            let is_shuffle = r.read_u8()?;
            let children = if num_children > 0 {
                Self::read_recursive(r, num_children)?
            } else { Vec::new() };
            items.push(Self {
                segment_id, playlist_item_id, num_children, rs_type,
                loop_val, loop_min, loop_max, weight, avoid_repeat_count,
                is_using_weight, is_shuffle, children,
            });
        }
        Ok(items)
    }
    fn write_recursive(items: &[Self], w: &mut BinaryWriter) {
        for item in items {
            w.write_u32(item.segment_id);
            w.write_u32(item.playlist_item_id);
            w.write_u32(item.num_children);
            w.write_u32(item.rs_type);
            w.write_i16(item.loop_val);
            w.write_i16(item.loop_min);
            w.write_i16(item.loop_max);
            w.write_u32(item.weight);
            w.write_u16(item.avoid_repeat_count);
            w.write_u8(item.is_using_weight);
            w.write_u8(item.is_shuffle);
            if item.num_children > 0 {
                Self::write_recursive(&item.children, w);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct MusicRanSeqValues {
    pub music_trans_node_params: MusicTransNodeParams,
    pub num_playlist_items: u32,
    pub playlist: Vec<MusicRanSeqPlaylistItem>,
}

impl MusicRanSeqValues {
    pub fn read(r: &mut BinaryReader, has_feedback: bool) -> Result<Self> {
        let music_trans_node_params = MusicTransNodeParams::read(r, has_feedback)?;
        let num_playlist_items = r.read_u32()?;
        let playlist = MusicRanSeqPlaylistItem::read_recursive(r, 1)?;
        Ok(Self { music_trans_node_params, num_playlist_items, playlist })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        self.music_trans_node_params.write(w);
        w.write_u32(self.num_playlist_items);
        MusicRanSeqPlaylistItem::write_recursive(&self.playlist, w);
    }
}

