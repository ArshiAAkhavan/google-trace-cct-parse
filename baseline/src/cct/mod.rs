use std::fmt::Display;

use log::{info, warn};

use crate::{Event, EventPhase};

#[cfg(test)]
mod verify;

mod visualize;

/// CCT is the struct that holds the Calling Context Tree.
/// each CCT consists of CCTMeta and a vector of CCTNodes.
#[derive(Default, Clone)]
pub struct CCT {
    nodes: Vec<CCTNode>,
    metadata: CCTMeta,
}

/// CCTNode is the representation of each context in the calling context tree.
/// each node has an id, a parent id which points to its parent in the CCT.
/// each node has an start and stop timestamp and holds the event from which the node
/// is created.
/// for event types that represent an instant in time, start and stop are equal.
#[derive(Debug, Clone)]
pub struct CCTNode {
    id: usize,
    start_time: i64,
    stop_time: Option<i64>,
    parent_node_id: Option<usize>,
    event: Event,
}

/// CCTMeta holds metadata of the tree.
/// currently only process name and thread name are supported.
#[derive(Debug, Clone, Default)]
pub struct CCTMeta {
    process_name: Option<String>,
    thread_name: Option<String>,
}

impl CCTNode {
    fn new(
        id: usize,
        start_time: i64,
        stop_time: Option<i64>,
        parent_node_id: Option<usize>,
        event: Event,
    ) -> Self {
        Self {
            id,
            start_time,
            stop_time,
            parent_node_id,
            event,
        }
    }
}
impl CCT {
    /// creates a new CCT and allocates its first node as root.
    fn new() -> Self {
        let root = CCTNode::new(
            0,
            i64::min_value(),
            Some(i64::max_value()),
            None,
            Default::default(),
        );
        Self {
            nodes: vec![root],
            ..Default::default()
        }
    }

    /// returns a refrence to the tree's root node.
    fn root(&self) -> &CCTNode {
        &self.nodes[0]
    }

    /// allocates a new node in the tree with the given parent_id as parent and returns a
    /// refrence to it.
    fn new_node(
        &mut self,
        start_time: i64,
        stop_time: Option<i64>,
        parent: Option<usize>,
        event: Event,
    ) -> &CCTNode {
        let node = CCTNode::new(self.nodes.len(), start_time, stop_time, parent, event);
        self.nodes.push(node);
        self.nodes.last().unwrap()
    }

    /// takes the node id and returns an immutable refrence to the node.
    fn get_node(&self, id: usize) -> &CCTNode {
        &self.nodes[id]
    }

    /// takes the node id and returns a mutable refrence to the node.
    fn get_node_mut(&mut self, id: usize) -> &mut CCTNode {
        &mut self.nodes[id]
    }
}

impl CCT {
    /// creates the cct from a vector of events.
    fn from_events(events: Vec<Event>) -> Self {
        let mut cct = CCT::new();
        let mut stack = Vec::with_capacity(events.len() / 2);
        stack.push(cct.root().id);

        // some nodes are made from instant events or duration events which represent a full
        // node instead of half of a node. when poping the event stack to get a handle to the
        // parent node, we should check if the parent node has a valid stop_timestamp and if,
        // check if the stop_timestamp is bigger than the event.timestamp
        fn pop_until_valid_parent<'a>(
            cct: &'a CCT,
            event_stack: &mut Vec<usize>,
            event: &Event,
        ) -> &'a CCTNode {
            let mut parent = cct.get_node(*event_stack.last().unwrap());

            // check for `EventPhase::Complete`s since this nodes have their stop time
            // available at construction
            while let Some(stop_time) = parent.stop_time {
                if stop_time <= event.timestamp {
                    event_stack.pop();
                    parent = cct.get_node(*event_stack.last().unwrap());
                } else {
                    break;
                }
            }
            parent
        }

        for mut event in events {
            match event.phase_type {
                EventPhase::SyncBegin | EventPhase::AsyncBegin | EventPhase::ObjectCreate => {
                    // create half of a node, set its parent, and push it into stack
                    let parent = pop_until_valid_parent(&cct, &mut stack, &event);
                    stack.push(
                        cct.new_node(event.timestamp, None, Some(parent.id), event)
                            .id,
                    );
                }

                EventPhase::SyncEnd | EventPhase::AsyncEnd | EventPhase::ObjectDestroy => {
                    // pop a half node from stack and complete it.
                    let id = loop {
                        match stack.pop() {
                            Some(id) => {
                                if cct.get_node(id).stop_time.is_some() {
                                    continue;
                                }
                                break id;
                            }
                            None => {
                                warn!("found event {event:#?} with no matching start");
                                continue;
                            }
                        }
                    };

                    let node = cct.get_node_mut(id);
                    node.stop_time = Some(event.timestamp);
                    node.event.merge(&mut event);
                }

                EventPhase::SyncInstant
                | EventPhase::AsyncInstant
                | EventPhase::ObjectSnapshot
                | EventPhase::MemoryDumpProcess
                | EventPhase::MemoryDumpGlobal
                | EventPhase::Mark => {
                    // create a full node and set its parent
                    let parent = pop_until_valid_parent(&cct, &mut stack, &event);
                    cct.new_node(
                        event.timestamp,
                        Some(event.timestamp),
                        Some(parent.id),
                        event,
                    );
                }
                EventPhase::Complete => {
                    // create a full node and push into the stack
                    let parent = pop_until_valid_parent(&cct, &mut stack, &event);
                    let node_id = cct
                        .new_node(
                            event.timestamp,
                            event.duration.or(Some(0)).map(|dur| dur + event.timestamp),
                            Some(parent.id),
                            event,
                        )
                        .id;
                    stack.push(node_id);
                }
                EventPhase::Counter
                | EventPhase::Sample
                | EventPhase::Clock
                | EventPhase::FlowStart
                | EventPhase::FlowStep
                | EventPhase::ContextEnter
                | EventPhase::ContextLeave
                | EventPhase::FlowEnd => ignored(&event),
                EventPhase::Metadata => {
                    // update CCT metadata
                    let name = extract_name_from_args(&event);
                    match &*event.name {
                        "process_name" => cct.metadata.process_name = Some(name),
                        "thread_name" => cct.metadata.thread_name = Some(name),
                        _ => (),
                    }
                }
            }
        }
        cct
    }

    /// normalize timestamps to be more human readable.
    /// this function substracts the root's start timestamp from all nodes
    /// to make the periods more human readable.
    pub fn normalize(&mut self) -> &Self {
        let time_shift = self
            .nodes
            .iter()
            .skip(1)
            .map(|node| node.start_time)
            .min()
            .unwrap_or_default();
        let max_time = self
            .nodes
            .iter()
            .skip(1)
            .map(|node| node.stop_time)
            .max()
            .unwrap_or(Some(i64::max_value()));

        self.nodes.iter_mut().for_each(|node| {
            if node.id == 0 {
                node.start_time = time_shift;
                node.stop_time = max_time;
            }
            node.start_time -= time_shift;
            node.stop_time = node.stop_time.map(|t| t - time_shift).or(max_time);
        });

        self
    }
}

impl From<Vec<Event>> for CCT {
    fn from(events: Vec<Event>) -> Self {
        CCT::from_events(events)
    }
}

/// tries to extract name field from a json map
/// returns a empty string if none is found
fn extract_name_from_args(event: &Event) -> String {
    if let Some(args) = &event.args {
        if let serde_json::Value::Object(obj) = args {
            let name = obj.get("name");
            if let Some(name) = name {
                if let serde_json::Value::String(name) = name {
                    return String::from(name);
                }
            }
        }
    }
    return String::from("");
}

fn ignored(event: &Event) {
    info!("event of phase {} is ignored", event.phase_type)
}

impl<'a> IntoIterator for &'a CCT {
    type Item = &'a CCTNode;

    type IntoIter = std::slice::Iter<'a, CCTNode>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}

impl std::fmt::Display for CCT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tree = visualize::build_visual_tree(self);
        let lines = visualize::visualize_tree(&tree, 160);
        writeln!(
            f,
            "context meta: {}|{}",
            &self.metadata.process_name.clone().unwrap_or_default(),
            &self.metadata.thread_name.clone().unwrap_or_default()
        )?;
        for line in lines {
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for CCT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let time_shift = self
            .nodes
            .iter()
            .skip(1)
            .map(|node| node.start_time)
            .min()
            .unwrap_or_default();
        writeln!(f, "time shift: {time_shift}")?;
        for mut node in self.nodes.clone().into_iter().skip(1) {
            node.start_time -= time_shift;
            node.stop_time = node.stop_time.map(|t| t - time_shift);
            writeln!(f, "{node}")?
        }
        Ok(())
    }
}

impl Display for CCTNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:*<45} <{}>::{}",
            format!(
                "[{}] -> [{}]: ({},{}) ",
                match self.parent_node_id {
                    Some(id) => id,
                    None => 0,
                },
                self.id,
                self.start_time,
                self.stop_time.unwrap_or(-1)
            ),
            self.event.name,
            self.event.phase_type,
        )
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

    use crate::{build_application_cct, cct::verify, collect_traces, Trace};

    /// ensures that the tree constraint holds, i.e.,
    /// for each pair of nodes (N1,N2) | N1 is an ancestor of N2 <==> N1 period encapsulates N2
    /// period
    #[test]
    fn check_cct_ordering() -> std::io::Result<()> {
        let trace: Trace = collect_traces(Path::new("../data/trace-valid-ending.json"))?;
        let app_cct = build_application_cct(trace);
        app_cct
            .sync_tasks
            .par_iter()
            .for_each(|(_, cct)| verify::assert_cct_valid(&cct));
        app_cct
            .async_tasks
            .par_iter()
            .for_each(|(_, cct)| verify::assert_cct_valid(&cct));
        app_cct
            .object_life_cycle
            .par_iter()
            .for_each(|(_, cct)| verify::assert_cct_valid(&cct));
        Ok(())
    }
}
