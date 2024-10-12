use crate::{cct::CCTNode, CCT};

pub fn assert_cct_valid(cct: &CCT) {
    for node1 in cct.into_iter() {
        for node2 in cct.into_iter() {
            if covers(node1, node2) && covers(node2, node1) {
                assert!(
                    is_node1_parent_of_node2(cct, node1, node2) || is_node1_parent_of_node2(cct, node2, node1),
                    "{node1}\n and \n{node2}\n have similar timestamps but none is parent of the other!"
                );
            } else if covers(node1, node2) {
                assert!(
                    is_node1_parent_of_node2(cct, node1, node2),
                    "{node1}\n is not parent of\n{node2}"
                );
            } else if covers(node2, node1) {
                assert!(
                    is_node1_parent_of_node2(cct, node2, node1),
                    "{node2}\n is not parent of\n{node1}"
                );
            }
        }
    }
}

fn is_node1_parent_of_node2(cct: &CCT, node1: &CCTNode, node2: &CCTNode) -> bool {
    if node1.id == node2.id {
        return true;
    }
    let mut child = node2;
    let parent = node1;
    while let Some(parent_id) = child.parent_node_id {
        if parent_id == parent.id {
            return true;
        }
        child = cct.get_node(parent_id);
    }
    false
}

fn covers(node1: &CCTNode, node2: &CCTNode) -> bool {
    if let Some(node2_stop_time) = node2.stop_time {
        if node2.start_time == node2_stop_time && node1.start_time == node2.start_time {
            return false;
        }
    }
    node1.start_time <= node2.start_time
        && match node1.stop_time {
            Some(self_stop_time) => match node2.stop_time {
                Some(other_stop_time) => {
                    self_stop_time >= other_stop_time && node2.start_time < self_stop_time
                }
                None => true,
            },
            None => true,
        }
}
