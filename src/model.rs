use crate::tshark::PacketInfo;
use indexmap::IndexMap;

/// A network node (endpoint) in the signaling diagram.
#[derive(Debug, Clone)]
pub struct Node {
    pub address: String,
}

/// A message (arrow) between two nodes.
#[derive(Debug, Clone)]
pub struct Message {
    pub frame_number: u32,
    pub timestamp: f64,
    pub source_idx: usize,
    pub dest_idx: usize,
    pub protocol: String,
    pub info: String,
}

/// The complete signaling diagram: an ordered set of nodes and messages.
#[derive(Debug)]
pub struct SignalingDiagram {
    pub nodes: Vec<Node>,
    pub messages: Vec<Message>,
}

impl SignalingDiagram {
    /// Build a signaling diagram from parsed packet info.
    /// Nodes are discovered in order of first appearance.
    pub fn from_packets(packets: &[PacketInfo]) -> Self {
        let mut node_map: IndexMap<String, usize> = IndexMap::new();
        let mut nodes = Vec::new();
        let mut messages = Vec::new();

        for packet in packets {
            let src_idx = *node_map.entry(packet.src.clone()).or_insert_with(|| {
                let idx = nodes.len();
                nodes.push(Node {
                    address: packet.src.clone(),
                });
                idx
            });

            let dst_idx = *node_map.entry(packet.dst.clone()).or_insert_with(|| {
                let idx = nodes.len();
                nodes.push(Node {
                    address: packet.dst.clone(),
                });
                idx
            });

            messages.push(Message {
                frame_number: packet.frame_number,
                timestamp: packet.time_relative,
                source_idx: src_idx,
                dest_idx: dst_idx,
                protocol: packet.protocol.clone(),
                info: packet.info.clone(),
            });
        }

        SignalingDiagram { nodes, messages }
    }
}
