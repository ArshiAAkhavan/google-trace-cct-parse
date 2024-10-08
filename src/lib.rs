mod cct;
mod trace;

use std::{
    cmp::max,
    collections::{hash_map::Entry, HashMap},
};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use trace::{ApplicationTrace, Event};

pub use cct::CallingContextTree;
pub use trace::Trace;

pub fn build_application_trace(trace: Trace) -> ApplicationTrace {
    let mut traces: HashMap<(i32, i32), Vec<Event>> = HashMap::new();
    trace
        .events
        .iter()
        .map(|e| ((e.pid, e.tid), e))
        .for_each(|(key, val)| {
            traces.entry(key).or_default().push(val.clone());
        });
    for (_, events) in traces.iter_mut() {
        events.sort();
    }

    todo!()
}
