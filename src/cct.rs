use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CallingContextTree {
    nodes: Vec<CCTNode>,
}

#[derive(Debug, Deserialize)]
pub struct CCTNode {
    start_time: i32,
    stop_time: i32,
    parent_node_id: Option<i32>,
}
