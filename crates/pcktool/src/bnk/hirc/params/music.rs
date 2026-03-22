//! Music system parameter types.

use super::super::reader::BinaryReader;
use super::super::writer::BinaryWriter;
use super::node::{Children, NodeBaseParams};
use super::rtpc::RtpcGraphPoint;
use crate::error::Result;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct MeterInfo {
    pub grid_period: f64,
    pub grid_offset: f64,
    pub tempo: f32,
    pub time_sig_num_beats_bar: u8,
    pub time_sig_beat_value: u8,
}

#[derive(Debug, Clone)]
pub struct Stinger {
    pub trigger_id: u32,
    pub segment_id: u32,
    pub sync_play_at: u32,
    pub cue_filter_hash: u32,
    pub dont_repeat_time: i32,
    pub num_segment_look_ahead: u32,
}

#[derive(Debug, Clone)]
pub struct MusicNodeParams {
    pub flags: u8,
    pub node_base_params: NodeBaseParams,
    pub children: Children,
    pub meter_info: MeterInfo,
    pub meter_info_flag: u8,
    pub stingers: Vec<Stinger>,
}

impl MusicNodeParams {
    pub fn read(r: &mut BinaryReader, has_feedback: bool) -> Result<Self> {
        let flags = r.read_u8()?;
        let node_base_params = NodeBaseParams::read(r, has_feedback)?;
        let children = Children::read(r)?;
        let meter_info = MeterInfo {
            grid_period: r.read_f64()?,
            grid_offset: r.read_f64()?,
            tempo: r.read_f32()?,
            time_sig_num_beats_bar: r.read_u8()?,
            time_sig_beat_value: r.read_u8()?,
        };
        let meter_info_flag = r.read_u8()?;
        let stingers = r.read_list_u32(|r| {
            Ok(Stinger {
                trigger_id: r.read_u32()?,
                segment_id: r.read_u32()?,
                sync_play_at: r.read_u32()?,
                cue_filter_hash: r.read_u32()?,
                dont_repeat_time: r.read_i32()?,
                num_segment_look_ahead: r.read_u32()?,
            })
        })?;
        Ok(Self {
            flags,
            node_base_params,
            children,
            meter_info,
            meter_info_flag,
            stingers,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.flags);
        self.node_base_params.write(w);
        self.children.write(w);
        w.write_f64(self.meter_info.grid_period);
        w.write_f64(self.meter_info.grid_offset);
        w.write_f32(self.meter_info.tempo);
        w.write_u8(self.meter_info.time_sig_num_beats_bar);
        w.write_u8(self.meter_info.time_sig_beat_value);
        w.write_u8(self.meter_info_flag);
        w.write_list_u32(&self.stingers, |w, s| {
            w.write_u32(s.trigger_id);
            w.write_u32(s.segment_id);
            w.write_u32(s.sync_play_at);
            w.write_u32(s.cue_filter_hash);
            w.write_i32(s.dont_repeat_time);
            w.write_u32(s.num_segment_look_ahead);
        });
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MusicFade {
    pub transition_time: i32,
    pub fade_curve: u32,
    pub fade_offset: i32,
}

impl MusicFade {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            transition_time: r.read_i32()?,
            fade_curve: r.read_u32()?,
            fade_offset: r.read_i32()?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_i32(self.transition_time);
        w.write_u32(self.fade_curve);
        w.write_i32(self.fade_offset);
    }
}

#[derive(Debug, Clone)]
pub struct MusicTransitionObject {
    pub segment_id: u32,
    pub fade_in: MusicFade,
    pub fade_out: MusicFade,
    pub play_pre_entry: u8,
    pub play_post_exit: u8,
}

impl MusicTransitionObject {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            segment_id: r.read_u32()?,
            fade_in: MusicFade::read(r)?,
            fade_out: MusicFade::read(r)?,
            play_pre_entry: r.read_u8()?,
            play_post_exit: r.read_u8()?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u32(self.segment_id);
        self.fade_in.write(w);
        self.fade_out.write(w);
        w.write_u8(self.play_pre_entry);
        w.write_u8(self.play_post_exit);
    }
}

#[derive(Debug, Clone)]
pub struct MusicTransSrcRule {
    pub transition_time: i32,
    pub fade_curve: u32,
    pub fade_offset: i32,
    pub sync_type: u32,
    pub cue_filter_hash: u32,
    pub play_post_exit: u8,
}

#[derive(Debug, Clone)]
pub struct MusicTransDstRule {
    pub transition_time: i32,
    pub fade_curve: u32,
    pub fade_offset: i32,
    pub cue_filter_hash: u32,
    pub jump_to_id: u32,
    pub entry_type: u16,
    pub play_pre_entry: u8,
    pub dest_match_source_cue_name: u8,
}

#[derive(Debug, Clone)]
pub struct MusicTransitionRule {
    pub src_ids: Vec<u32>,
    pub dst_ids: Vec<u32>,
    pub src_rule: MusicTransSrcRule,
    pub dst_rule: MusicTransDstRule,
    pub alloc_trans_object_flag: u8,
    pub trans_object: Option<MusicTransitionObject>,
}

impl MusicTransitionRule {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let src_ids = r.read_list_u32(|r| r.read_u32())?;
        let dst_ids = r.read_list_u32(|r| r.read_u32())?;
        let src_rule = MusicTransSrcRule {
            transition_time: r.read_i32()?,
            fade_curve: r.read_u32()?,
            fade_offset: r.read_i32()?,
            sync_type: r.read_u32()?,
            cue_filter_hash: r.read_u32()?,
            play_post_exit: r.read_u8()?,
        };
        let dst_rule = MusicTransDstRule {
            transition_time: r.read_i32()?,
            fade_curve: r.read_u32()?,
            fade_offset: r.read_i32()?,
            cue_filter_hash: r.read_u32()?,
            jump_to_id: r.read_u32()?,
            entry_type: r.read_u16()?,
            play_pre_entry: r.read_u8()?,
            dest_match_source_cue_name: r.read_u8()?,
        };
        let flag = r.read_u8()?;
        let trans_object = if flag != 0 {
            Some(MusicTransitionObject::read(r)?)
        } else {
            None
        };
        Ok(Self {
            src_ids,
            dst_ids,
            src_rule,
            dst_rule,
            alloc_trans_object_flag: flag,
            trans_object,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_list_u32(&self.src_ids, |w, &id| w.write_u32(id));
        w.write_list_u32(&self.dst_ids, |w, &id| w.write_u32(id));
        w.write_i32(self.src_rule.transition_time);
        w.write_u32(self.src_rule.fade_curve);
        w.write_i32(self.src_rule.fade_offset);
        w.write_u32(self.src_rule.sync_type);
        w.write_u32(self.src_rule.cue_filter_hash);
        w.write_u8(self.src_rule.play_post_exit);
        w.write_i32(self.dst_rule.transition_time);
        w.write_u32(self.dst_rule.fade_curve);
        w.write_i32(self.dst_rule.fade_offset);
        w.write_u32(self.dst_rule.cue_filter_hash);
        w.write_u32(self.dst_rule.jump_to_id);
        w.write_u16(self.dst_rule.entry_type);
        w.write_u8(self.dst_rule.play_pre_entry);
        w.write_u8(self.dst_rule.dest_match_source_cue_name);
        w.write_u8(self.alloc_trans_object_flag);
        if let Some(obj) = &self.trans_object {
            obj.write(w);
        }
    }
}

#[derive(Debug, Clone)]
pub struct MusicTransNodeParams {
    pub music_node_params: MusicNodeParams,
    pub rules: Vec<MusicTransitionRule>,
}

impl MusicTransNodeParams {
    pub fn read(r: &mut BinaryReader, has_feedback: bool) -> Result<Self> {
        let music_node_params = MusicNodeParams::read(r, has_feedback)?;
        let rules = r.read_list_u32(MusicTransitionRule::read)?;
        Ok(Self {
            music_node_params,
            rules,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        self.music_node_params.write(w);
        w.write_list_u32(&self.rules, |w, rule| rule.write(w));
    }
}

#[derive(Debug, Clone)]
pub struct TrackSrcInfo {
    pub track_id: u32,
    pub source_id: u32,
    pub play_at: f64,
    pub begin_trim_offset: f64,
    pub end_trim_offset: f64,
    pub src_duration: f64,
}

impl TrackSrcInfo {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            track_id: r.read_u32()?,
            source_id: r.read_u32()?,
            play_at: r.read_f64()?,
            begin_trim_offset: r.read_f64()?,
            end_trim_offset: r.read_f64()?,
            src_duration: r.read_f64()?,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u32(self.track_id);
        w.write_u32(self.source_id);
        w.write_f64(self.play_at);
        w.write_f64(self.begin_trim_offset);
        w.write_f64(self.end_trim_offset);
        w.write_f64(self.src_duration);
    }
}

#[derive(Debug, Clone)]
pub struct ClipAutomation {
    pub clip_index: u32,
    pub auto_type: u32,
    pub graph_points: Vec<RtpcGraphPoint>,
}

impl ClipAutomation {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let clip_index = r.read_u32()?;
        let auto_type = r.read_u32()?;
        let graph_points = r.read_list_u32(RtpcGraphPoint::read)?;
        Ok(Self {
            clip_index,
            auto_type,
            graph_points,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u32(self.clip_index);
        w.write_u32(self.auto_type);
        w.write_list_u32(&self.graph_points, |w, p| p.write(w));
    }
}

#[derive(Debug, Clone)]
pub struct TrackSwitchParams {
    pub group_type: u8,
    pub group_id: u32,
    pub default_switch: u32,
    pub switch_associations: Vec<u32>,
}

impl TrackSwitchParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let group_type = r.read_u8()?;
        let group_id = r.read_u32()?;
        let default_switch = r.read_u32()?;
        let switch_associations = r.read_list_u32(|r| r.read_u32())?;
        Ok(Self {
            group_type,
            group_id,
            default_switch,
            switch_associations,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u8(self.group_type);
        w.write_u32(self.group_id);
        w.write_u32(self.default_switch);
        w.write_list_u32(&self.switch_associations, |w, &v| w.write_u32(v));
    }
}

#[derive(Debug, Clone)]
pub struct TrackTransParams {
    pub src_fade_params: MusicFade,
    pub sync_type: u32,
    pub cue_filter_hash: u32,
    pub dest_fade_params: MusicFade,
}

impl TrackTransParams {
    pub fn read(r: &mut BinaryReader) -> Result<Self> {
        let src_fade_params = MusicFade::read(r)?;
        let sync_type = r.read_u32()?;
        let cue_filter_hash = r.read_u32()?;
        let dest_fade_params = MusicFade::read(r)?;
        Ok(Self {
            src_fade_params,
            sync_type,
            cue_filter_hash,
            dest_fade_params,
        })
    }
    pub fn write(&self, w: &mut BinaryWriter) {
        self.src_fade_params.write(w);
        w.write_u32(self.sync_type);
        w.write_u32(self.cue_filter_hash);
        self.dest_fade_params.write(w);
    }
}
