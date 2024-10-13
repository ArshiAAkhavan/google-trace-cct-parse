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

fn covers(parent: &CCTNode, child: &CCTNode) -> bool {
    match parent.stop_time {
        Some(parent_stop_time) => match child.stop_time {
            Some(child_stop_time) => {
                // node2 is an instant event that started with the range event
                // so they can be two seperate events
                // s1 == s2 == e2
                if parent.start_time == child.start_time && child.start_time == child_stop_time {
                    false
                } else {
                    parent.start_time < child.start_time && child_stop_time < parent_stop_time
                }
            }
            None => parent.start_time < child.start_time && child.start_time < parent_stop_time,
        },
        None => parent.start_time < child.start_time,
    }
}
