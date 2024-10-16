mod application;
mod cct;
mod trace;

use std::fs::File;
use std::io::BufReader;
use std::io::Result;
use std::path::Path;

pub use application::ApplicationCCT;
use application::ApplicationTrace;

pub use cct::CCT;

pub use trace::{Category, Id, ProcessId, Scope, ThreadId};
pub use trace::{Event, EventPhase, Trace};

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
                let id = (event.pid, event.tid);
                app_trace.sync_tasks.entry(id).or_default().push(event);
            }
            EventPhase::AsyncBegin | EventPhase::AsyncEnd | EventPhase::AsyncInstant => {
                let id = (event.scope.clone(), event.id, event.category.clone());
                app_trace.async_tasks.entry(id).or_default().push(event);
            }
            EventPhase::ObjectCreate | EventPhase::ObjectSnapshot | EventPhase::ObjectDestroy => {
                let id = (event.scope.clone(), event.id);
                app_trace
                    .object_life_cycle
                    .entry(id)
                    .or_default()
                    .push(event);
            }
            _ => (),
        }
    }
    app_trace.application_cct()
}
