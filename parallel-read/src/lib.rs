use std::io::Result;
use std::path::Path;

mod read;
use baseline::ApplicationCCT;
use baseline::Trace;
use read::parallel_read;

pub fn collect_traces(trace_path: &Path) -> Result<Trace> {
    parallel_read(trace_path)
}

pub fn build_application_cct(trace: Trace) -> ApplicationCCT {
    baseline::build_application_cct(trace)
}
