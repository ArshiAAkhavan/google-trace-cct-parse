mod application;
mod cct;
mod trace;

use std::fs::File;
use std::io::BufReader;
use std::io::Result;
use std::path::Path;

pub use trace::ApplicationCCT;
use trace::ApplicationTrace;

pub use cct::CCT;

pub use trace::Event;
pub use trace::EventPhase;
pub use trace::Trace;

pub fn collect_traces(trace_path: &Path) -> Result<Trace> {
    let data = File::open(trace_path)?;
    let data = BufReader::new(data);
    let trace: Trace = serde_json::from_reader(data)?;
    Ok(trace)
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
