use std::{
    fs::File,
    io::{BufRead, BufReader, Result},
    path::Path,
};

use application::ApplicationTrace;
use baseline::{ApplicationCCT, Event, EventPhase};
use log::debug;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

mod application;
mod read;

/// build_application_cct converts ApplicationTrace into ApplicationCCT
pub fn build_application_cct(app_trace: ApplicationTrace) -> ApplicationCCT {
    app_trace.application_cct()
}

/// collect_traces reads a tracefile and construct an ApplicationTrace
pub fn collect_traces(file_path: &Path) -> Result<ApplicationTrace> {
    let num_threads = rayon::current_num_threads();
    debug!("concurrency level: {num_threads}");

    // read file size
    let file = File::open(file_path)?;
    let file_size = file.metadata()?.len();

    // ignore the first line since it represents the trace array,
    // i.e., {"traceEvents":[
    let init_skip = BufReader::new(file).read_until(b'\n', &mut vec![]).unwrap();

    // calculate chunksize
    let chunk_size = (file_size as usize - init_skip + num_threads - 1) / num_threads;

    // create thread ids
    let threads: Vec<usize> = (0..num_threads).collect();

    let application_trace = threads
        .par_iter()
        .filter_map(move |thread_id| {
            read::collect_events(*thread_id, file_path, chunk_size, init_skip as u64).ok()
        })
        .map(|events| build_application_trace(events))
        .reduce(Default::default, |mut first, second| {
            for (id, mut events) in second.sync_tasks.into_iter() {
                first.sync_tasks.entry(id).or_default().append(&mut events)
            }
            for (id, mut events) in second.async_tasks.into_iter() {
                first.async_tasks.entry(id).or_default().append(&mut events)
            }
            for (id, mut events) in second.object_life_cycle.into_iter() {
                first
                    .object_life_cycle
                    .entry(id)
                    .or_default()
                    .append(&mut events)
            }
            first
        });

    Ok(application_trace)
}

/// creates an ApplicationTrace from a vector of events
fn build_application_trace(events: Vec<Event>) -> ApplicationTrace {
    let mut app_trace = ApplicationTrace::new();
    for event in events.into_iter() {
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
    app_trace
}
