use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Result;
use std::path::Path;

use application::ApplicationCCT;
use application::ApplicationTrace;
use baseline::Event;
use baseline::EventPhase;
use log::debug;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;

mod application;
mod read;

pub fn build_application_cct(app_trace: ApplicationTrace) -> ApplicationCCT {
    app_trace.application_cct()
}

pub fn collect_traces(file_path: &Path) -> Result<ApplicationTrace> {
    let num_threads = rayon::current_num_threads();
    debug!("concurrency level: {num_threads}");

    let file = File::open(file_path)?;
    let file_size = file.metadata()?.len();

    // ignore the first line since it represents the trace array,
    // i.e., {"traceEvents":[
    let init_skip = BufReader::new(file).read_until(b'\n', &mut vec![]).unwrap();

    let chunk_size = (file_size as usize - init_skip + num_threads - 1) / num_threads;

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
