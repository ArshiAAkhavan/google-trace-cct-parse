mod cct;
mod trace;
mod utils;

use std::collections::HashMap;

use trace::{ApplicationTrace, ThreadTrace};

pub use cct::CallingContextTree as CCT;
pub use trace::Event;
pub use trace::EventPhase;
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
    let mut app = ApplicationTrace::new();
    for ((pid, tid), events) in traces.into_iter() {
        app.processes.entry(pid).and_modify(|process_trace| {
            process_trace
                .threads
                .entry(tid)
                .or_insert(ThreadTrace::new(tid, CCT::new(events)));
        });
    }

    todo!()
}
