use crate::Event;

#[derive(Debug)]
pub struct CallingContextTree {
    nodes: Vec<CCTNode>,
}

#[derive(Debug)]
pub struct CCTNode {
    start_time: i32,
    stop_time: i32,
    parent_node_id: Option<i32>,
}

impl CallingContextTree {
    pub fn new(events: Vec<Event>) -> Self {
        todo!()
    }
}
