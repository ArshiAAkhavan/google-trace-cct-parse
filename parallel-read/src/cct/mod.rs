use std::fmt::Display;

use log::{info, warn};

use crate::{Event, EventPhase};

#[cfg(test)]
mod verify;

mod visualize;

#[derive(Clone)]
pub struct CCT {
    nodes: Vec<CCTNode>,
    metadata: CCTMeta,
}

#[derive(Debug, Clone)]
pub struct CCTNode {
    id: usize,
    start_time: i64,
    stop_time: Option<i64>,
    parent_node_id: Option<usize>,
    event: Event,
}

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
            metadata: Default::default(),
        }
    }
    fn root(&self) -> &CCTNode {
        &self.nodes[0]
    }

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

        for event in events {
            match event.phase_type {
                EventPhase::SyncBegin | EventPhase::AsyncBegin | EventPhase::ObjectCreate => {
                    let parent = pop_until_valid_parent(&cct, &mut event_stack, &event);
                    event_stack.push(
                        cct.new_node(event.timestamp, None, Some(parent.id), event)
                            .id,
                    );
                }
                EventPhase::SyncEnd | EventPhase::AsyncEnd | EventPhase::ObjectDestroy => {
                    let id = loop {
                        match event_stack.pop() {
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
                    node.event.merge(&event)
                }
                EventPhase::SyncInstant
                | EventPhase::AsyncInstant
                | EventPhase::ObjectSnapshot
                | EventPhase::MemoryDumpProcess
                | EventPhase::MemoryDumpGlobal
                | EventPhase::Mark => {
                    let parent = pop_until_valid_parent(&cct, &mut event_stack, &event);
                    cct.new_node(
                        event.timestamp,
                        Some(event.timestamp),
                        Some(parent.id),
                        event,
                    );
                }
                EventPhase::Complete => {
                    let parent = pop_until_valid_parent(&cct, &mut event_stack, &event);
                    let node_id = cct
                        .new_node(
                            event.timestamp,
                            event.duration.or(Some(0)).map(|dur| dur + event.timestamp),
                            Some(parent.id),
                            event,
                        )
                        .id;
                    event_stack.push(node_id);
                }
                EventPhase::Counter
                | EventPhase::Sample
                | EventPhase::Clock
                | EventPhase::FlowStart
                | EventPhase::FlowStep
                | EventPhase::FlowEnd => ignored(event),
                EventPhase::Metadata => {
                    let name = extract_name_from_args(&event);
                    match &*event.name {
                        "process_name" => cct.metadata.process_name = Some(name),
                        "thread_name" => cct.metadata.thread_name = Some(name),
                        _ => (),
                    }
                }
                EventPhase::ContextEnter => todo!(),
                EventPhase::ContextLeave => todo!(),
            }
        }
        cct
    }
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

        // Initialize TreeNodes
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

fn ignored(event: Event) {
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
