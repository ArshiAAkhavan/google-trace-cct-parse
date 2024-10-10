use log::warn;

use crate::{Event, EventPhase};

#[derive(Debug)]
pub struct CCT {
    nodes: Vec<CCTNode>,
}

#[derive(Debug)]
pub struct CCTNode {
    id: usize,
    start_time: i64,
    stop_time: i64,
    parent_node_id: Option<usize>,
}

impl CCTNode {
    fn new(id: usize, start_time: i64, stop_time: i64, parent_node_id: Option<usize>) -> Self {
        Self {
            id,
            start_time,
            stop_time,
            parent_node_id,
        }
    }
}
impl CCT {
    fn new() -> Self {
        let root = CCTNode::new(0, i64::min_value(), i64::max_value(), None);
        Self { nodes: vec![root] }
    }
    fn root(&self) -> &CCTNode {
        &self.nodes[0]
    }

    fn new_node(&mut self, start_time: i64, stop_time: i64, parent: Option<usize>) -> &CCTNode {
        let node = CCTNode::new(self.nodes.len(), start_time, stop_time, parent);
        self.nodes.push(node);
        self.nodes.last().unwrap()
    }

    fn get_node(&self, id: usize) -> &CCTNode {
        &self.nodes[id]
    }
    fn get_node_mut(&mut self, id: usize) -> &mut CCTNode {
        &mut self.nodes[id]
    }
}

impl CCT {
    pub fn from_events(events: Vec<Event>) -> Self {
        let mut cct = CCT::new();

        let mut event_stack = vec![cct.root().id];
        for event in events {
            match event.phase_type {
                EventPhase::SyncBegin | EventPhase::AsyncBegin => event_stack.push(
                    cct.new_node(
                        event.timestamp,
                        0,
                        Some(cct.get_node(*event_stack.last().unwrap()).id),
                    )
                    .id,
                ),
                EventPhase::SyncEnd | EventPhase::AsyncEnd => {
                    let node = cct.get_node_mut(match event_stack.pop() {
                        Some(id) => id,
                        None => {
                            warn!("found event {event:#?} with no matching start");
                            continue;
                        }
                    });
                    node.stop_time = event.timestamp;
                }
                EventPhase::SyncInstant | EventPhase::AsyncInstant => {
                    cct.new_node(
                        event.timestamp,
                        event.timestamp,
                        Some(*event_stack.last().unwrap()),
                    );
                }
                _ => (),
            }
        }
        cct
    }
}
