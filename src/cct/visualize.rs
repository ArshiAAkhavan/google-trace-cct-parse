use super::CCT;

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

struct VisualNode {
    id: usize,
    start: i64,
    end: i64,
    children: Vec<usize>,
}

pub fn visualize_tree(tree: &VisualTree, max_char: usize) -> Vec<String> {
    let root = &tree.nodes[0];
    let mut lines = Vec::new();
    lines.push(format!("|{:-^width$}|", "#0#", width = max_char - 2));
    visualize(&root, tree, max_char, 0, &mut lines, 1);
    lines
}

fn visualize(
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
    let ranges: Vec<(f32, usize)> = ranges
        .iter()
        .map(|(s, e, id)| (f32::sqrt((e - s) as f32), *id))
        .collect();

    let id_overhead: usize = root.children.iter().map(|id| format!("#{id}#").len()).sum();
    let factor: f32 =
        (len.saturating_sub(id_overhead)) as f32 / ranges.iter().map(|(w, _)| *w).sum::<f32>();

    let mut children_repr = String::new();
    for (weight, id) in &ranges {
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
