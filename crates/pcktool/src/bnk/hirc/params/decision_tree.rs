//! Decision tree types for DialogueEvent and MusicSwitch.

use alloc::vec::Vec;
use crate::error::Result;
use super::super::reader::BinaryReader;
use super::super::writer::BinaryWriter;

/// A node in a decision tree (AkDecisionTree).
#[derive(Debug, Clone)]
pub struct DecisionTreeNode {
    pub key: u32,
    pub is_audio_node: bool,
    pub audio_node_id: u32,
    pub children_idx: u16,
    pub children_count: u16,
    pub weight: u16,
    pub probability: u16,
    pub children: Vec<DecisionTreeNode>,
}

impl DecisionTreeNode {
    pub fn write(&self, w: &mut BinaryWriter) {
        w.write_u32(self.key);
        if self.is_audio_node {
            w.write_u32(self.audio_node_id);
        } else {
            let packed = (self.children_idx as u32) | ((self.children_count as u32) << 16);
            w.write_u32(packed);
        }
        w.write_u16(self.weight);
        w.write_u16(self.probability);
    }
}

/// Decision tree used in MusicSwitch and DialogueEvent.
#[derive(Debug, Clone)]
pub struct DecisionTree {
    pub mode: u8,
    pub root_nodes: Vec<DecisionTreeNode>,
    pub raw_tree_data: Option<Vec<u8>>,
}

impl DecisionTree {
    pub fn read(r: &mut BinaryReader, depth: u32, tree_data_size: u32) -> Result<Self> {
        const ITEM_SIZE: u32 = 0x0c;
        let count_max = tree_data_size / ITEM_SIZE;
        let mut root_nodes = Vec::new();

        if count_max == 0 {
            return Ok(Self { mode: 0, root_nodes, raw_tree_data: None });
        }

        let start_pos = r.position();
        match Self::parse_nodes(r, 1, count_max, 0, depth) {
            Ok(nodes) => {
                root_nodes = nodes;
                Ok(Self { mode: 0, root_nodes, raw_tree_data: None })
            }
            Err(_) => {
                let consumed = r.position() - start_pos;
                let remaining = tree_data_size as usize - consumed;
                let raw = r.read_bytes(remaining)?.to_vec();
                Ok(Self { mode: 0, root_nodes: Vec::new(), raw_tree_data: Some(raw) })
            }
        }
    }

    fn parse_nodes(
        r: &mut BinaryReader,
        count: u32,
        count_max: u32,
        cur_depth: u32,
        max_depth: u32,
    ) -> Result<Vec<DecisionTreeNode>> {
        let mut node_infos: Vec<(DecisionTreeNode, u16)> = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let key = r.read_u32()?;
            let next_val = r.read_u32()?;
            let uidx = (next_val & 0xFFFF) as u16;
            let ucnt = ((next_val >> 16) & 0xFFFF) as u16;
            let is_audio = uidx as u32 > count_max
                || ucnt as u32 > count_max
                || cur_depth == max_depth;

            let weight = r.read_u16()?;
            let probability = r.read_u16()?;

            let node = if is_audio {
                DecisionTreeNode {
                    key, is_audio_node: true, audio_node_id: next_val,
                    children_idx: 0, children_count: 0,
                    weight, probability, children: Vec::new(),
                }
            } else {
                DecisionTreeNode {
                    key, is_audio_node: false, audio_node_id: 0,
                    children_idx: uidx, children_count: ucnt,
                    weight, probability, children: Vec::new(),
                }
            };
            let child_count = if is_audio { 0 } else { ucnt };
            node_infos.push((node, child_count));
        }

        let mut nodes = Vec::with_capacity(node_infos.len());
        for (mut node, child_count) in node_infos {
            if child_count > 0 {
                node.children = Self::parse_nodes(
                    r, child_count as u32, count_max, cur_depth + 1, max_depth,
                )?;
            }
            nodes.push(node);
        }
        Ok(nodes)
    }

    pub fn write(&self, w: &mut BinaryWriter, _depth: u32) {
        if let Some(raw) = &self.raw_tree_data {
            w.write_bytes(raw);
            return;
        }
        let mut all_nodes: Vec<&DecisionTreeNode> = Vec::new();
        Self::collect_nodes(&self.root_nodes, &mut all_nodes);

        let mut flat: Vec<DecisionTreeNode> = all_nodes.iter().map(|n| (*n).clone()).collect();
        let mut current_index: u16 = 1;
        for node in &mut flat {
            if !node.is_audio_node && !node.children.is_empty() {
                node.children_idx = current_index;
                node.children_count = node.children.len() as u16;
                current_index += node.children.len() as u16;
            }
        }
        for node in &flat {
            node.write(w);
        }
    }

    fn collect_nodes<'a>(nodes: &'a [DecisionTreeNode], out: &mut Vec<&'a DecisionTreeNode>) {
        out.extend(nodes.iter());
        for node in nodes {
            if !node.children.is_empty() {
                Self::collect_nodes(&node.children, out);
            }
        }
    }
}

