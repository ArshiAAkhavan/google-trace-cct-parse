use std::io::Result;
use std::path::Path;
use trace::ApplicationTrace;

pub use trace::{ApplicationCCT, Event, EventPhase, Trace};

mod cct;
mod read;
mod trace;
pub use cct::CCT;
use read::parallel_read;

pub fn collect_traces(trace_path: &Path) -> Result<Trace> {
    parallel_read(trace_path)
}

pub fn build_application_cct(trace: Trace) -> ApplicationCCT {
    let mut app_trace = ApplicationTrace::new();

    for event in trace.events.into_iter() {
        match event.phase_type {
            EventPhase::SyncBegin
            | EventPhase::SyncEnd
            | EventPhase::SyncInstant
            | EventPhase::Complete => {
                let task_id = (event.pid, event.tid);
                app_trace
                    .sync_tasks
                    .entry(task_id)
                    .and_modify(|events| events.push(event.clone()))
                    .or_insert(vec![event]);
            }
            EventPhase::AsyncBegin | EventPhase::AsyncEnd | EventPhase::AsyncInstant => {
                let task_id = (event.scope.clone(), event.id, event.category.clone());
                app_trace
                    .async_tasks
                    .entry(task_id)
                    .and_modify(|events| events.push(event.clone()))
                    .or_insert(vec![event]);
            }
            EventPhase::ObjectCreate | EventPhase::ObjectSnapshot | EventPhase::ObjectDestroy => {
                let object_lifecycle_id = (event.scope.clone(), event.id);
                app_trace
                    .object_life_cycle
                    .entry(object_lifecycle_id)
                    .and_modify(|events| events.push(event.clone()))
                    .or_insert(vec![event]);
            }
            _ => (),
        }
    }
    app_trace.application_cct()
}
