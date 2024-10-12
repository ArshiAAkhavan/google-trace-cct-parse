use std::fmt::Display;

use log::{info, warn};

use crate::{Event, EventPhase};

#[derive(Debug)]
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
    ) -> Self {
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
        let root = CCTNode::new(0, i64::min_value(), Some(i64::max_value()), None);
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
    ) -> &CCTNode {
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
                    event_stack.push(cct.new_node(event.timestamp, None, Some(parent.id)).id);
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
                }
                EventPhase::SyncInstant
                | EventPhase::AsyncInstant
                | EventPhase::ObjectSnapshot
                | EventPhase::MemoryDumpProcess
                | EventPhase::MemoryDumpGlobal
                | EventPhase::Mark => {
                    let parent = pop_until_valid_parent(&cct, &mut event_stack, &event);
                    cct.new_node(event.timestamp, Some(event.timestamp), Some(parent.id));
                }
                EventPhase::Complete => {
                    let parent = pop_until_valid_parent(&cct, &mut event_stack, &event);
                    let node_id = cct
                        .new_node(
                            event.timestamp,
                            event.duration.or(Some(0)).map(|dur| dur + event.timestamp),
                            Some(parent.id),
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

impl Display for CCT {
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
            "[{}] -> [{}]: ({},{})",
            match self.parent_node_id {
                Some(id) => id,
                None => 0,
            },
            self.id,
            self.start_time,
            self.stop_time.unwrap_or_default()
        )
    }
}

fn ignored(event: Event) {
    info!("event of phase {} is ignored", event.phase_type)
}

pub struct VisualTree {
    nodes: Vec<VisualNode>,
}

pub fn build_visual_tree(cct: &CCT) -> VisualTree {
    let min_time = cct
        .nodes
        .iter()
        .skip(1)
        .map(|n| n.start_time)
        .min()
        .unwrap();
    let max_time = cct
        .nodes
        .iter()
        .skip(1)
        .map(|n| n.stop_time.unwrap_or_default())
        .max()
        .unwrap();
    let mut tree = VisualTree { nodes: Vec::new() };
    tree.nodes.push(VisualNode {
        id: 0,
        start: min_time,
        end: max_time,
        children: Vec::new(),
    });
    for node in cct.nodes.iter().skip(1) {
        tree.nodes.push(VisualNode {
            id: node.id,
            start: node.start_time,
            end: node.stop_time.unwrap_or(node.start_time),
            children: Vec::new(),
        })
    }

    for node in cct.nodes.iter().skip(1) {
        tree.nodes[node.parent_node_id.unwrap()]
            .children
            .push(node.id)
    }
    tree
}

pub struct VisualNode {
    id: usize,
    start: i64,
    end: i64,
    children: Vec<usize>,
}

pub fn visualize_tree(tree: &VisualTree, max_char: usize) {
    let root = &tree.nodes[0];
    let mut lines = Vec::new();
    visualize(&root, tree, max_char, 0, &mut lines, 0);
    println!("|{:-^width$}|", "#0#", width = max_char - 2);
    for line in &lines {
        println!("{line}");
    }
}

pub fn visualize(
    root: &VisualNode,
    tree: &VisualTree,
    len: usize,
    index: usize,
    lines: &mut Vec<String>,
    h: usize,
) {
    if root.children.len() == 0 {
        return;
    }
    if h >= lines.len() {
        lines.push(String::from(""));
    }

    let mut ranges = Vec::new();
    for child_id in &root.children {
        let child = &tree.nodes[*child_id];
        ranges.push((child.start, child.end, child.id))
    }
    ranges.sort_by_key(|(start, _, _)| *start);
    let range_weights: Vec<(f32, usize)> = ranges
        .iter()
        .map(|(s, e, id)| (f32::sqrt((e - s) as f32), *id))
        .collect();

    let id_char_len: usize = root.children.iter().map(|id| format!("#{id}#").len()).sum();
    let factor: f32 = (len.saturating_sub(id_char_len)) as f32
        / range_weights.iter().map(|(w, _)| *w).sum::<f32>();

    let mut children_repr = String::new();
    for (weight, id) in &range_weights {
        let child_len = ((weight * factor) as usize).saturating_sub(1);
        let child_index = index + children_repr.len() + 1;
        let child_repr = format!(
            "{:-^width$}",
            format!("#{}#", id),
            width = child_len.saturating_sub(1)
        );
        children_repr.push('|');
        children_repr.push_str(&child_repr);
        let root = &tree.nodes[*id];

        visualize(root, tree, child_len, child_index, lines, h + 1);
    }
    children_repr.push('|');
    let children_repr = format!("{: <width$}", children_repr, width = len);
    if lines[h].len() < index {
        let lines_len = lines[h].len();
        lines[h].push_str(&format!(
            "{: <width$}",
            "",
            width = (index - lines_len).saturating_sub(1)
        ));
    }
    lines[h].push_str(&children_repr);
}
